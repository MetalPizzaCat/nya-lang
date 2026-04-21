use core::panic;
use std::collections::HashMap;

use crate::{
    garbage_collect::{GarbageCollector, GcInnerGuard, GcInnerGuardMut, GcObject},
    instruction::Instruction,
    object::{FromNyaType, IntoNyaType, Nil, NyaHeapType, NyaPrimitiveType},
};

fn calc_idx(len: usize, idx: isize) -> usize {
    (if idx < 0 { len as isize + idx } else { idx } as usize)
}

/// This type holds the state of the virtual machine
pub struct NyaState {
    gc: GarbageCollector,
    stack: Vec<Vec<NyaPrimitiveType>>,
    globals: HashMap<String, NyaPrimitiveType>,
    variables: Vec<Vec<NyaPrimitiveType>>,
    constant_pool: Vec<NyaPrimitiveType>,
}

impl NyaState {
    /// Create a new NyaState
    pub fn new() -> Self {
        Self {
            gc: GarbageCollector::new(),
            stack: Vec::new(),
            globals: HashMap::new(),
            variables: Vec::new(),
            constant_pool: Vec::new(),
        }
    }

    pub fn run_instructions(
        &mut self,
        arguments: Vec<NyaPrimitiveType>,
        instructions: &[Instruction],
    ) {
        let mut pc: usize = 0;
        self.create_variable_block();
        for i in 0..arguments.len() {
            self.set_local(i, arguments[i]);
        }
        self.create_stack_block();
        'exec: while pc < instructions.len() {
            match instructions[pc] {
                Instruction::Push(obj) => self.push_stack_object(obj),
                Instruction::Pop => self.pop_stack(1),
                Instruction::SetGlobal(name) => {
                    // INFO: I added a get_constant function that auto copies the inner value so you
                    // don't have to do a double deref
                    let Some(NyaPrimitiveType::HeapRef(name_str)) = self.get_constant(name) else {
                        panic!("Invalid constant id '{}'", name)
                    };
                    let NyaHeapType::String(name) = &*name_str.borrow() else {
                        panic!("Expected string on stack as global name")
                    };
                    self.pop_global(&name.clone());
                }
                Instruction::GetGlobal(name) => {
                    let Some(NyaPrimitiveType::HeapRef(name_str)) = self.get_constant(name) else {
                        panic!("Invalid constant id '{}'", name)
                    };
                    let NyaHeapType::String(name) = &*name_str.borrow() else {
                        panic!("Expected string on stack as global name")
                    };
                    self.push_global(&name.clone());
                }
                Instruction::Add => {
                    let Some(a) = self.pop_stack_and_take() else {
                        panic!("not enough items on the stack")
                    };
                    let Some(b) = self.pop_stack_and_take() else {
                        panic!("not enough items on the stack")
                    };
                    match (a, b) {
                        (NyaPrimitiveType::Int(a), NyaPrimitiveType::Int(b)) => {
                            self.push_value(NyaPrimitiveType::Int(a + b))
                        }
                        (NyaPrimitiveType::Number(a), NyaPrimitiveType::Int(b)) => {
                            self.push_value(NyaPrimitiveType::Number(a + b as f64))
                        }
                        (NyaPrimitiveType::Int(a), NyaPrimitiveType::Number(b)) => {
                            self.push_value(NyaPrimitiveType::Number(a as f64 + b))
                        }
                        (NyaPrimitiveType::Number(a), NyaPrimitiveType::Number(b)) => {
                            self.push_value(NyaPrimitiveType::Number(a + b))
                        }
                        (_, _) => panic!("types cannot be added"),
                    }
                }
                Instruction::Halt => break 'exec,
                Instruction::GetLocal(id) => {
                    let Some(val) = self.get_local(id) else {
                        panic!("No variable with id {} exists", id)
                    };
                    self.push_value(*val)
                }
                Instruction::SetLocal(id) => {
                    let Some(val) = self.pop_stack_and_take() else {
                        panic!("Missing value on stack")
                    };
                    self.set_local(id, val);
                }
                Instruction::GetConst(id) => self.push_constant(id),
                Instruction::Print => {
                    if let Some(val) = self.pop_stack_and_take() {
                        println!(
                            "{}",
                            match val {
                                NyaPrimitiveType::Number(val) => val.to_string(),
                                NyaPrimitiveType::Int(val) => val.to_string(),
                                NyaPrimitiveType::Nil => "nil".to_owned(),
                                NyaPrimitiveType::HeapRef(obj) => match &mut obj.borrow().clone() {
                                    NyaHeapType::Table(_) => "invalid type".to_owned(),
                                    NyaHeapType::String(s) => s.clone(),
                                },
                            }
                        );
                    }
                }
            }
            pc += 1
        }
    }

    pub fn create_variable_block(&mut self) {
        self.variables.push(Vec::new());
    }

    pub fn create_stack_block(&mut self) {
        self.stack.push(Vec::new());
    }

    pub fn set_local(&mut self, id: usize, val: NyaPrimitiveType) {
        let Some(block) = self.variables.last_mut() else {
            panic!("Missing variable block")
        };
        if block.len() <= id {
            block.resize(id + 1, NyaPrimitiveType::Nil);
        }
        block[id] = val;
        // this is where i would increase ref count btw, but this doesn't use that
    }

    pub fn get_local(&self, id: usize) -> Option<&NyaPrimitiveType> {
        // INFO: if you don't care a about the error message this can be changed to
        // `self.variables.last()?.get(id)`
        let Some(block) = self.variables.last() else {
            panic!("Missing variable block")
        };
        block.get(id)
    }

    // fetching data

    pub fn get_number(&self, idx: isize) -> Option<f64> {
        if let Some(NyaPrimitiveType::Number(number)) = self.get_stack_object(idx) {
            Some(number)
        } else {
            None
        }
    }

    pub fn get_number_mut(&mut self, idx: isize) -> Option<&mut f64> {
        if let Some(NyaPrimitiveType::Number(number)) = self.get_stack_object_mut(idx) {
            Some(number)
        } else {
            None
        }
    }

    pub fn get_int(&self, idx: isize) -> Option<i64> {
        if let Some(NyaPrimitiveType::Int(i)) = self.get_stack_object(idx) {
            Some(i)
        } else {
            None
        }
    }

    pub fn get_int_mut(&mut self, idx: isize) -> Option<&mut i64> {
        if let Some(NyaPrimitiveType::Int(i)) = self.get_stack_object_mut(idx) {
            Some(i)
        } else {
            None
        }
    }

    pub fn get_string(&self, idx: isize) -> Option<GcInnerGuard<String>> {
        self.get_stack(idx)
    }

    pub fn get_string_mut(&mut self, idx: isize) -> Option<GcInnerGuardMut<String>> {
        self.get_stack(idx)
    }

    pub fn get_stack<T>(&self, stack_idx: isize) -> Option<T>
    where
        T: FromNyaType,
    {
        if let Some(obj) = self.get_stack_object(stack_idx) {
            T::from_nya_object(obj)
        } else {
            None
        }
    }

    pub fn push_field<T>(&mut self, stack_idx: isize, field: T)
    where
        T: IntoNyaType,
    {
        if let Some(NyaPrimitiveType::HeapRef(heap_obj)) = self.get_stack_object(stack_idx)
            && let NyaHeapType::Table(table) = &*heap_obj.borrow()
            && let Some(key) = field.into_nya_object(self).into_hashable()
            && let Some(obj) = table.get(&key)
        {
            self.push_stack_object(*obj);
        } else {
            self.push_value(Nil);
        }
    }

    pub fn pop_field(&mut self, stack_idx: isize, field: &str) {
        if let Some(NyaPrimitiveType::HeapRef(heap_obj)) = self.get_stack_object(stack_idx)
            && let NyaHeapType::Table(table) = &mut *heap_obj.borrow_mut()
            && let Some(key) = field.into_nya_object(self).into_hashable()
        {
            if let Some(obj) = self.pop_stack_and_take() {
                table.insert(key, obj);
            } else {
                table.insert(key, Nil.into_nya_object(self));
            }
        }
    }

    // memory

    /// Allocate an object on the gc heap. If it is not in a root
    pub fn alloc_heap_object(&mut self, obj: NyaHeapType) -> GcObject {
        self.gc.alloc(obj)
    }

    pub fn add_constant<T>(&mut self, obj: T) -> usize
    where
        T: IntoNyaType,
    {
        let obj = obj.into_nya_object(self);
        self.constant_pool.push(obj);
        self.constant_pool.len() - 1
    }

    pub fn get_constant(&mut self, idx: usize) -> Option<NyaPrimitiveType> {
        self.constant_pool.get(idx).copied()
    }

    pub fn push_constant(&mut self, idx: usize) {
        if let Some(obj) = self.constant_pool.get(idx) {
            self.push_stack_object(*obj);
        } else {
            self.push_value(Nil);
        }
    }

    fn get_stack_object(&self, idx: isize) -> Option<NyaPrimitiveType> {
        let Some(stack) = self.stack.last() else {
            panic!("No stack is available");
        };
        let idx = calc_idx(stack.len(), idx);
        stack.get(idx).copied()
    }

    fn get_stack_object_mut(&mut self, idx: isize) -> Option<&mut NyaPrimitiveType> {
        let id = calc_idx(self.stack.len(), idx);
        let Some(stack) = self.stack.last_mut() else {
            panic!("No stack is available");
        };
        stack.get_mut(id)
    }

    fn push_stack_object(&mut self, obj: NyaPrimitiveType) {
        let Some(stack) = self.stack.last_mut() else {
            panic!("No stack is available");
        };
        stack.push(obj);
    }

    pub fn push_value<T>(&mut self, object: T)
    where
        T: IntoNyaType,
    {
        let obj = object.into_nya_object(self);
        self.push_stack_object(obj);
    }

    fn pop_stack_and_take(&mut self) -> Option<NyaPrimitiveType> {
        let current_stack = self.stack.last_mut()?;
        current_stack.pop()
    }

    pub fn pop_stack(&mut self, n: usize) {
        for _ in 0..n {
            self.pop_stack_and_take();
        }
    }

    pub fn set_global_direct<T>(&mut self, name: &str, object: T)
    where
        T: IntoNyaType,
    {
        let obj = object.into_nya_object(self);
        self.globals.insert(name.to_string(), obj);
    }

    pub fn remove_global(&mut self, name: &str) {
        self.globals.remove(name);
    }

    pub fn push_global(&mut self, name: &str) {
        self.push_stack_object(
            self.globals
                .get(name)
                .map_or(NyaPrimitiveType::Nil, |obj| *obj),
        );
    }

    pub fn pop_global(&mut self, name: &str) {
        let obj = self
            .pop_stack_and_take()
            .map_or(NyaPrimitiveType::Nil, |obj| obj);
        self.set_global_direct(name, obj);
    }

    pub fn garbage_collect(&mut self) {
        for stack in &mut self.stack {
            self.gc.mark_slice(stack);
        }

        for obj in self.globals.values_mut() {
            self.gc.mark(obj);
        }

        self.gc.mark_slice(&mut self.constant_pool);

        unsafe {
            self.gc.sweep();
        }
    }
}
