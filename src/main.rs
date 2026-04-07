use crate::nya_object::{NyaHeapObject, NyaObject};

pub mod nya_object;

struct NyaState {
    stack: Vec<NyaObject>,
    heap: Vec<NyaHeapObject>,
}

impl NyaState {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            heap: Vec::new(),
        }
    }

    fn create_obj(&mut self, obj: NyaObject) -> NyaHeapObject {
        let heap_obj = unsafe { NyaHeapObject::new(obj) };
        self.heap.push(heap_obj.clone());
        heap_obj
    }

    fn push_stack_move(&mut self, obj: NyaObject) {
        self.stack.push(obj);
    }

    pub fn push_number(&mut self, number: f64) {
        self.push_stack_move(NyaObject::Number(number));
    }

    fn pop_stack_take(&mut self) -> Option<NyaObject> {
        self.stack.pop()
    }

    pub fn pop_stack(&mut self) {
        self.pop_stack_take();
    }

    pub fn pop_number(&mut self) -> Option<f64> {
        self.pop_stack_take().and_then(|obj| match obj {
            NyaObject::Number(n) => Some(n),
            _ => None,
        })
    }

    pub fn garbage_collect(&mut self) {
        for obj in &mut self.heap {
            obj.marked = false;
        }

        for obj in &mut self.stack {
            obj.mark_children();
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

fn main() {
    let mut ns = NyaState::new();
    ns.garbage_collect();
    ns.push_number(0.5);
    ns.garbage_collect();
    let n = ns.pop_number();
    println!("{n:?}");
    ns.garbage_collect();
}
