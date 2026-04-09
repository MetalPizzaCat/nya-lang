use std::collections::HashMap;

use crate::{
    instruction::Instruction,
    object::{IntoNyaType, Nil, NyaHeapObject, NyaHeapType, NyaPrimativeType},
};

fn calc_idx(len: usize, idx: isize) -> usize {
    (if idx < 0 { len as isize + idx } else { idx } as usize)
}

/// This type holds the state of the virtual machine
pub struct NyaState {
    stack: Vec<NyaPrimativeType>,
    heap: Vec<NyaHeapObject>,
    globals: HashMap<String, NyaPrimativeType>,
}

impl NyaState {
    /// Create a new NyaState
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            heap: Vec::new(),
            globals: HashMap::new(),
        }
    }

    pub fn run_instructions(&mut self, instructions: &[Instruction]) {
        for instruction in instructions {
            match instruction {
                Instruction::Push(obj) => self.push_stack_object(*obj),
                Instruction::Pop => self.pop_stack(1),
                Instruction::SetGlobal(name) => self.set_global(name),
                Instruction::GetGlobal(name) => self.get_global(&name),
                Instruction::Add => {
                    let Some(a) = self.pop_stack_and_take() else {
                        panic!("not enough items on the stack")
                    };
                    let Some(b) = self.pop_stack_and_take() else {
                        panic!("not enough items on the stack")
                    };
                    match (a, b) {
                        (NyaPrimativeType::Int(a), NyaPrimativeType::Int(b)) => {
                            self.push_value(NyaPrimativeType::Int(a + b))
                        }
                        (NyaPrimativeType::Number(a), NyaPrimativeType::Int(b)) => {
                            self.push_value(NyaPrimativeType::Number(a + b as f64))
                        }
                        (NyaPrimativeType::Int(a), NyaPrimativeType::Number(b)) => {
                            self.push_value(NyaPrimativeType::Number(a as f64 + b))
                        }
                        (NyaPrimativeType::Number(a), NyaPrimativeType::Number(b)) => {
                            self.push_value(NyaPrimativeType::Number(a + b))
                        }
                        (_, _) => panic!("types cannot be added"),
                    }
                }
                Instruction::Halt => return,
            }
        }
    }

    // fetching data

    pub fn get_number(&self, idx: isize) -> Option<f64> {
        if let Some(NyaPrimativeType::Number(number)) = self.get_stack(idx) {
            Some(*number)
        } else {
            None
        }
    }

    pub fn get_number_mut(&mut self, idx: isize) -> Option<&mut f64> {
        if let Some(NyaPrimativeType::Number(number)) = self.get_stack_mut(idx) {
            Some(number)
        } else {
            None
        }
    }

    pub fn get_int(&self, idx: isize) -> Option<i64> {
        if let Some(NyaPrimativeType::Int(i)) = self.get_stack(idx) {
            Some(*i)
        } else {
            None
        }
    }

    pub fn get_int_mut(&mut self, idx: isize) -> Option<&mut i64> {
        if let Some(NyaPrimativeType::Int(i)) = self.get_stack_mut(idx) {
            Some(i)
        } else {
            None
        }
    }

    pub fn get_string(&self, idx: isize) -> Option<&str> {
        if let Some(NyaPrimativeType::HeapRef(heap_obj)) = self.get_stack(idx)
            && let NyaHeapType::String(s) = &***heap_obj
        {
            Some(s)
        } else {
            None
        }
    }

    pub fn get_string_mut(&mut self, idx: isize) -> Option<&mut String> {
        if let Some(NyaPrimativeType::HeapRef(heap_obj)) = self.get_stack_mut(idx)
            && let NyaHeapType::String(s) = &mut ***heap_obj
        {
            Some(s)
        } else {
            None
        }
    }

    pub fn get_index(&mut self, stack_idx: isize, idx: isize) {
        if let Some(NyaPrimativeType::HeapRef(heap_obj)) = self.get_stack(stack_idx)
            && let NyaHeapType::Array(array) = &***heap_obj
            && let Some(obj) = array.get(calc_idx(array.len(), idx))
        {
            self.push_stack_object(*obj);
        } else {
            self.push_value(Nil);
        }
    }

    pub fn set_index(&mut self, stack_idx: isize, idx: isize) {
        if let Some(NyaPrimativeType::HeapRef(heap_obj)) = self.get_stack(stack_idx)
            && let NyaHeapType::Array(array) = &mut *(*heap_obj.clone())
        {
            if let Some(obj) = self.pop_stack_and_take() {
                array.push(obj);
            } else {
                array.push(Nil.into_nya_object(self));
            }
        }
    }

    pub fn get_field(&mut self, stack_idx: isize, field: &str) {
        if let Some(NyaPrimativeType::HeapRef(heap_obj)) = self.get_stack(stack_idx)
            && let NyaHeapType::Table(array) = &***heap_obj
            && let Some(obj) = array.get(field)
        {
            self.push_stack_object(*obj);
        } else {
            self.push_value(Nil);
        }
    }

    pub fn set_field(&mut self, stack_idx: isize, field: &str) {
        if let Some(NyaPrimativeType::HeapRef(heap_obj)) = self.get_stack(stack_idx)
            && let NyaHeapType::Table(table) = &mut *(*heap_obj.clone())
        {
            if let Some(obj) = self.pop_stack_and_take() {
                table.insert(field.to_string(), obj);
            } else {
                table.insert(field.to_string(), Nil.into_nya_object(self));
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

    fn get_stack(&self, idx: isize) -> Option<&NyaPrimativeType> {
        let idx = calc_idx(self.stack.len(), idx);
        self.stack.get(idx)
    }

    fn get_stack_mut(&mut self, idx: isize) -> Option<&mut NyaPrimativeType> {
        let idx = calc_idx(self.stack.len(), idx);
        self.stack.get_mut(idx)
    }

    fn push_stack_object(&mut self, obj: NyaPrimativeType) {
        self.stack.push(obj);
    }

    pub fn push_value<T>(&mut self, object: T)
    where
        T: IntoNyaType,
    {
        let obj = object.into_nya_object(self);
        self.push_stack_object(obj);
    }

    fn pop_stack_and_take(&mut self) -> Option<NyaPrimativeType> {
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
                .map_or(NyaPrimativeType::Nil, |obj| *obj),
        );
    }

    pub fn set_global(&mut self, name: &str) {
        let obj = self
            .pop_stack_and_take()
            .map_or(NyaPrimativeType::Nil, |obj| obj);
        self.set_global_direct(name, obj);
    }

    pub fn garbage_collect(&mut self) {
        for obj in &mut self.heap {
            obj.marked = false;
        }

        for obj in &mut self.stack {
            if let NyaPrimativeType::HeapRef(obj) = obj {
                obj.marked = true;
                obj.mark_children();
            }
        }

        for obj in self.globals.values_mut() {
            if let NyaPrimativeType::HeapRef(obj) = obj {
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
