use core::panic;
use std::collections::HashMap;

use crate::{
    garbage_collector::{GarbageCollector, GcInnerGuard, GcInnerGuardMut, GcObject},
    instruction::Instruction,
    object::{FromNyaObject, IntoNyaObject, Nil, NyaHeapObject, NyaPrimitiveObject},
};

/// calculate index from a index that can be negative for the end of the array.
///
/// # Examples
/// ```rust,ignore
/// assert_eq!(calc_idx(10, -1), 9);
/// ```
fn calc_idx(len: usize, idx: isize) -> usize {
    (if idx < 0 { len as isize + idx } else { idx } as usize)
}

/// This type holds the state of the virtual machine
pub struct NyaState {
    gc: GarbageCollector,
    stack: Vec<Vec<NyaPrimitiveObject>>,
    globals: HashMap<String, NyaPrimitiveObject>,
    variables: Vec<Vec<NyaPrimitiveObject>>,
    constant_pool: Vec<NyaPrimitiveObject>,
}

impl NyaState {
    /// Create a new NyaState with a new stack and variable frame by default.
    pub fn new() -> Self {
        Self {
            gc: GarbageCollector::new(),
            stack: Vec::new(),
            globals: HashMap::new(),
            variables: Vec::new(),
            constant_pool: Vec::new(),
        }
    }

    /// Run instructions.
    pub fn run_instructions(
        &mut self,
        arguments: Vec<NyaPrimitiveObject>,
        instructions: &[Instruction],
    ) {
        let mut pc: usize = 0;
        self.create_variable_frame();
        for i in 0..arguments.len() {
            self.set_local(i, arguments[i]);
        }
        self.create_stack_frame();
        'exec: while pc < instructions.len() {
            match instructions[pc] {
                Instruction::Push(obj) => self.push_stack_object(obj),
                Instruction::Pop => self.pop_stack(1),
                Instruction::SetGlobal(name) => {
                    // INFO: I added a get_constant function that auto copies the inner value so you
                    // don't have to do a double deref
                    let Some(NyaPrimitiveObject::HeapRef(name_str)) = self.get_constant(name)
                    else {
                        panic!("Invalid constant id '{}'", name)
                    };
                    let NyaHeapObject::String(name) = &*name_str.borrow() else {
                        panic!("Expected string on stack as global name")
                    };
                    self.pop_global(&name.clone());
                }
                Instruction::GetGlobal(name) => {
                    let Some(NyaPrimitiveObject::HeapRef(name_str)) = self.get_constant(name)
                    else {
                        panic!("Invalid constant id '{}'", name)
                    };
                    let NyaHeapObject::String(name) = &*name_str.borrow() else {
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
                        (NyaPrimitiveObject::Int(a), NyaPrimitiveObject::Int(b)) => {
                            self.push_value(NyaPrimitiveObject::Int(a + b))
                        }
                        (NyaPrimitiveObject::Number(a), NyaPrimitiveObject::Int(b)) => {
                            self.push_value(NyaPrimitiveObject::Number(a + b as f64))
                        }
                        (NyaPrimitiveObject::Int(a), NyaPrimitiveObject::Number(b)) => {
                            self.push_value(NyaPrimitiveObject::Number(a as f64 + b))
                        }
                        (NyaPrimitiveObject::Number(a), NyaPrimitiveObject::Number(b)) => {
                            self.push_value(NyaPrimitiveObject::Number(a + b))
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
                                NyaPrimitiveObject::Number(val) => val.to_string(),
                                NyaPrimitiveObject::Int(val) => val.to_string(),
                                NyaPrimitiveObject::Nil => "nil".to_owned(),
                                NyaPrimitiveObject::HeapRef(obj) => match &mut obj.borrow().clone()
                                {
                                    NyaHeapObject::Table(_) => "invalid type".to_owned(),
                                    NyaHeapObject::String(s) => s.clone(),
                                },
                            }
                        );
                    }
                }
            }
            pc += 1
        }
    }

    /// Create a new variable frame. Functions will start interacting with the new frame by
    /// default.
    pub fn create_variable_frame(&mut self) {
        self.variables.push(Vec::new());
    }

    /// Create a new stack frame. All functions interacting with the stack will use the new frame.
    pub fn create_stack_frame(&mut self) {
        self.stack.push(Vec::new());
    }

    // Set a local variable in the current frame and create it if it doesn't exist.
    pub fn set_local(&mut self, id: usize, val: NyaPrimitiveObject) {
        let Some(block) = self.variables.last_mut() else {
            panic!("Missing variable block")
        };
        if block.len() <= id {
            block.resize(id + 1, NyaPrimitiveObject::Nil);
        }
        block[id] = val;
    }

    // Get a local variable in the current frame.
    pub fn get_local(&self, id: usize) -> Option<&NyaPrimitiveObject> {
        self.variables.last()?.get(id)
    }

    // pub fn get_number(&self, idx: isize) -> Option<f64> {
    //     if let Some(NyaPrimitiveObject::Number(number)) = self.get_stack_object(idx) {
    //         Some(number)
    //     } else {
    //         None
    //     }
    // }
    //
    // pub fn get_number_mut(&mut self, idx: isize) -> Option<&mut f64> {
    //     if let Some(NyaPrimitiveObject::Number(number)) = self.get_stack_object_mut(idx) {
    //         Some(number)
    //     } else {
    //         None
    //     }
    // }
    //
    // pub fn get_int(&self, idx: isize) -> Option<i64> {
    //     if let Some(NyaPrimitiveObject::Int(i)) = self.get_stack_object(idx) {
    //         Some(i)
    //     } else {
    //         None
    //     }
    // }
    //
    // pub fn get_int_mut(&mut self, idx: isize) -> Option<&mut i64> {
    //     if let Some(NyaPrimitiveObject::Int(i)) = self.get_stack_object_mut(idx) {
    //         Some(i)
    //     } else {
    //         None
    //     }
    // }
    //
    // pub fn get_string(&self, idx: isize) -> Option<GcInnerGuard<String>> {
    //     self.get_stack(idx)
    // }
    //
    // pub fn get_string_mut(&mut self, idx: isize) -> Option<GcInnerGuardMut<String>> {
    //     self.get_stack(idx)
    // }

    /// Push a value onto the current stack frame.
    pub fn push_value<T>(&mut self, object: T)
    where
        T: IntoNyaObject,
    {
        let obj = object.into_nya_object(self);
        self.push_stack_object(obj);
    }

    /// Pop a value from the current stack frame.
    pub fn pop_stack(&mut self, n: usize) {
        for _ in 0..n {
            self.pop_stack_and_take();
        }
    }

    /// Get a variable from the stack.
    pub fn get_stack<T>(&self, stack_idx: isize) -> Option<T>
    where
        T: FromNyaObject,
    {
        if let Some(obj) = self.get_stack_object(stack_idx) {
            T::from_nya_object(obj)
        } else {
            None
        }
    }

    /// Get field `field` from the table at `stack_idx` and push it onto the top of the stack.
    pub fn push_field<T>(&mut self, stack_idx: isize, field: T)
    where
        T: IntoNyaObject,
    {
        if let Some(NyaPrimitiveObject::HeapRef(heap_obj)) = self.get_stack_object(stack_idx)
            && let NyaHeapObject::Table(table) = &*heap_obj.borrow()
            && let Some(key) = field.into_nya_object(self).into_hashable()
            && let Some(obj) = table.get(&key)
        {
            self.push_stack_object(*obj);
        } else {
            self.push_value(Nil);
        }
    }

    /// Pop the object off the top of the stack and put it into `field` in the table at
    /// `stack_idx`.
    pub fn pop_field(&mut self, stack_idx: isize, field: &str) {
        if let Some(NyaPrimitiveObject::HeapRef(heap_obj)) = self.get_stack_object(stack_idx)
            && let NyaHeapObject::Table(table) = &mut *heap_obj.borrow_mut()
            && let Some(key) = field.into_nya_object(self).into_hashable()
        {
            if let Some(obj) = self.pop_stack_and_take() {
                table.insert(key, obj);
            } else {
                table.insert(key, Nil.into_nya_object(self));
            }
        }
    }

    /// Allocate an object on the [`GarbageCollector`] heap. See [`GarbageCollector::alloc`] for
    /// more.
    pub fn alloc_heap_object(&mut self, obj: NyaHeapObject) -> GcObject {
        self.gc.alloc(obj)
    }

    /// Add a constant to the constants pool returning its index.
    pub fn add_constant<T>(&mut self, obj: T) -> usize
    where
        T: IntoNyaObject,
    {
        let obj = obj.into_nya_object(self);
        self.constant_pool.push(obj);
        self.constant_pool.len() - 1
    }

    /// Get a constant from the constants pool by its index.
    pub fn get_constant(&mut self, idx: usize) -> Option<NyaPrimitiveObject> {
        self.constant_pool.get(idx).copied()
    }

    /// Push a constant from the constants pool onto the stack.
    pub fn push_constant(&mut self, idx: usize) {
        if let Some(obj) = self.constant_pool.get(idx) {
            self.push_stack_object(*obj);
        } else {
            self.push_value(Nil);
        }
    }

    /// Directly set a global by name a value.
    pub fn set_global_direct<T>(&mut self, name: &str, object: T)
    where
        T: IntoNyaObject,
    {
        let obj = object.into_nya_object(self);
        self.globals.insert(name.to_string(), obj);
    }

    /// Remove a global by name.
    pub fn remove_global(&mut self, name: &str) {
        self.globals.remove(name);
    }

    /// Push a global onto the stack.
    pub fn push_global(&mut self, name: &str) {
        self.push_stack_object(
            self.globals
                .get(name)
                .map_or(NyaPrimitiveObject::Nil, |obj| *obj),
        );
    }

    /// Pop value from the stack into a global.
    pub fn pop_global(&mut self, name: &str) {
        let obj = self
            .pop_stack_and_take()
            .map_or(NyaPrimitiveObject::Nil, |obj| obj);
        self.set_global_direct(name, obj);
    }

    /// Free all unused heap objects.
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

    // Internal functions.

    fn get_stack_object(&self, idx: isize) -> Option<NyaPrimitiveObject> {
        let Some(stack) = self.stack.last() else {
            panic!("No stack is available");
        };
        let idx = calc_idx(stack.len(), idx);
        stack.get(idx).copied()
    }

    fn get_stack_object_mut(&mut self, idx: isize) -> Option<&mut NyaPrimitiveObject> {
        let id = calc_idx(self.stack.len(), idx);
        let Some(stack) = self.stack.last_mut() else {
            panic!("No stack is available");
        };
        stack.get_mut(id)
    }

    fn push_stack_object(&mut self, obj: NyaPrimitiveObject) {
        let Some(stack) = self.stack.last_mut() else {
            panic!("No stack is available");
        };
        stack.push(obj);
    }

    fn pop_stack_and_take(&mut self) -> Option<NyaPrimitiveObject> {
        let current_stack = self.stack.last_mut()?;
        current_stack.pop()
    }
}
