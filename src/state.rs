use std::collections::HashMap;

use crate::{
    instruction::Instruction,
    object::{IntoNyaType, Nil, NyaHeapObject, NyaHeapType, NyaPrimitiveType},
};

fn calc_idx(len: usize, idx: isize) -> usize {
    (if idx < 0 { len as isize + idx } else { idx } as usize)
}

/// This type holds the state of the virtual machine
pub struct NyaState {
    stack: Vec<NyaPrimitiveType>,
    heap: Vec<NyaHeapObject>,
    globals: HashMap<String, NyaPrimitiveType>,
    constant_pool: Vec<NyaPrimitiveType>,
}

impl NyaState {
    /// Create a new NyaState
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            heap: Vec::new(),
            globals: HashMap::new(),
            constant_pool: Vec::new(),
        }
    }

    pub fn run_instructions(&mut self, instructions: &[Instruction]) {
        for instruction in instructions {
            match instruction {
                Instruction::Push(obj) => self.push_stack_object(*obj),
                Instruction::Pop => self.pop_stack(1),
                Instruction::SetGlobal(name) => self.set_global(name),
                Instruction::GetGlobal(name) => self.get_global(name),
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
                Instruction::Halt => return,
            }
        }
    }

    // fetching data

    pub fn to_number(&self, idx: isize) -> Option<f64> {
        if let Some(NyaPrimitiveType::Number(number)) = self.get_stack(idx) {
            Some(number)
        } else {
            None
        }
    }

    pub fn to_number_mut(&mut self, idx: isize) -> Option<&mut f64> {
        if let Some(NyaPrimitiveType::Number(number)) = self.get_stack_mut(idx) {
            Some(number)
        } else {
            None
        }
    }

    pub fn to_int(&self, idx: isize) -> Option<i64> {
        if let Some(NyaPrimitiveType::Int(i)) = self.get_stack(idx) {
            Some(i)
        } else {
            None
        }
    }

    pub fn to_int_mut(&mut self, idx: isize) -> Option<&mut i64> {
        if let Some(NyaPrimitiveType::Int(i)) = self.get_stack_mut(idx) {
            Some(i)
        } else {
            None
        }
    }

    pub fn to_string(&self, idx: isize) -> Option<String> {
        if let Some(NyaPrimitiveType::HeapRef(heap_obj)) = self.get_stack(idx)
            && let NyaHeapType::String(s) = (**heap_obj).clone()
        {
            Some(s)
        } else {
            None
        }
    }

    pub fn to_mut(&mut self, idx: isize) -> Option<&mut String> {
        if let Some(NyaPrimitiveType::HeapRef(heap_obj)) = self.get_stack_mut(idx)
            && let NyaHeapType::String(s) = &mut ***heap_obj
        {
            Some(s)
        } else {
            None
        }
    }

    pub fn get_field<T>(&mut self, stack_idx: isize, field: T)
    where
        T: IntoNyaType,
    {
        if let Some(NyaPrimitiveType::HeapRef(heap_obj)) = self.get_stack(stack_idx)
            && let NyaHeapType::Table(table) = &**heap_obj
            && let Some(key) = field.into_nya_object(self).into_hashable()
            && let Some(obj) = table.get(&key)
        {
            self.push_stack_object(*obj);
        } else {
            self.push_value(Nil);
        }
    }

    pub fn set_field(&mut self, stack_idx: isize, field: &str) {
        if let Some(NyaPrimitiveType::HeapRef(mut heap_obj)) = self.get_stack(stack_idx)
            && let NyaHeapType::Table(table) = &mut **heap_obj
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
    pub fn alloc_heap_object(&mut self, obj: NyaHeapType) -> NyaHeapObject {
        let heap_obj = unsafe { NyaHeapObject::new(obj) };
        self.heap.push(heap_obj);
        heap_obj
    }

    pub fn add_constant<T>(&mut self, obj: T) -> usize
    where
        T: IntoNyaType,
    {
        let obj = obj.into_nya_object(self);
        self.constant_pool.push(obj);
        self.constant_pool.len() - 1
    }

    pub fn get_constant(&mut self, idx: usize) {
        if let Some(obj) = self.constant_pool.get(idx) {
            self.push_stack_object(*obj);
        } else {
            self.push_value(Nil);
        }
    }

    fn get_stack(&self, idx: isize) -> Option<NyaPrimitiveType> {
        let idx = calc_idx(self.stack.len(), idx);
        self.stack.get(idx).copied()
    }

    fn get_stack_mut(&mut self, idx: isize) -> Option<&mut NyaPrimitiveType> {
        let idx = calc_idx(self.stack.len(), idx);
        self.stack.get_mut(idx)
    }

    fn push_stack_object(&mut self, obj: NyaPrimitiveType) {
        self.stack.push(obj);
    }

    pub fn push_value<T>(&mut self, object: T)
    where
        T: IntoNyaType,
    {
        let obj = object.into_nya_object(self);
        self.push_stack_object(obj);
    }

    fn pop_stack_and_take(&mut self) -> Option<NyaPrimitiveType> {
        self.stack.pop()
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

    pub fn get_global(&mut self, name: &str) {
        self.push_stack_object(
            self.globals
                .get(name)
                .map_or(NyaPrimitiveType::Nil, |obj| *obj),
        );
    }

    pub fn set_global(&mut self, name: &str) {
        let obj = self
            .pop_stack_and_take()
            .map_or(NyaPrimitiveType::Nil, |obj| obj);
        self.set_global_direct(name, obj);
    }

    pub fn garbage_collect(&mut self) {
        for obj in &mut self.heap {
            obj.marked = false;
        }

        for obj in &mut self.stack {
            if let NyaPrimitiveType::HeapRef(obj) = obj {
                obj.marked = true;
                obj.mark_children();
            }
        }

        for obj in self.globals.values_mut() {
            if let NyaPrimitiveType::HeapRef(obj) = obj {
                obj.marked = true;
                obj.mark_children();
            }
        }

        for obj in &mut self.constant_pool {
            if let NyaPrimitiveType::HeapRef(obj) = obj {
                obj.marked = true;
                obj.mark_children();
            }
        }

        for i in (0..self.heap.len()).rev() {
            if !self.heap[i].marked {
                let obj = self.heap.swap_remove(i);
                println!("freed {:?}", **obj);
                unsafe {
                    obj.free();
                }
            }
        }
    }
}

impl Drop for NyaState {
    fn drop(&mut self) {
        for obj in &self.heap {
            unsafe {
                obj.free();
            }
        }
    }
}
