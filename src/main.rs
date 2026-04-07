use std::collections::HashMap;

type HeapPointer = usize;

enum NyaObject {
    String(String),
    Number(f64),
    Table(HashMap<String, HeapPointer>),
    Nil,
}

struct NyaHeapObject {
    inner: NyaObject,
    marked: bool,
}

impl NyaHeapObject {
    pub fn mark_children(&self, heap: &mut Vec<Option<NyaHeapObject>>) {}
}

struct NyaState {
    stack: Vec<HeapPointer>,
    heap: Vec<Option<NyaHeapObject>>,
}

impl NyaState {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            heap: Vec::new(),
        }
    }

    pub fn collect_garbage(&mut self) {
        for ptr in &self.stack {
            let Some(obj) = &mut self.heap[*ptr] else {
                panic!("stack pointer points to nothing")
            };
            if !obj.marked {
                obj.marked = true;
            }
        }
    }
}

fn main() {}
