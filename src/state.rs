use crate::object::{IntoNyaObject, NyaHeapObject, NyaHeapType, NyaPrimativeType};

fn calc_idx(len: usize, idx: isize) -> usize {
    (if idx < 0 { len as isize + idx } else { idx } as usize)
}

pub struct NyaState {
    stack: Vec<NyaPrimativeType>,
    heap: Vec<NyaHeapObject>,
}

impl NyaState {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            heap: Vec::new(),
        }
    }

    pub fn alloc_heap_object(&mut self, obj: NyaHeapType) -> NyaHeapObject {
        let heap_obj = unsafe { NyaHeapObject::new(obj) };
        self.heap.push(heap_obj.clone());
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
        T: IntoNyaObject,
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
        if let Some(NyaPrimativeType::String(s)) = self.get_stack(idx) {
            Some(s)
        } else {
            None
        }
    }

    pub fn get_string_mut(&mut self, idx: isize) -> Option<&mut String> {
        if let Some(NyaPrimativeType::String(s)) = self.get_stack_mut(idx) {
            Some(s)
        } else {
            None
        }
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
                obj.clone().free();
            }
        }
    }
}
