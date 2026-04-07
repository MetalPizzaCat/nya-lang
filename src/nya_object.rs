use std::{
    alloc::{Layout, alloc, dealloc, handle_alloc_error},
    collections::HashMap,
    ops::{Deref, DerefMut},
};

/// This type holds a value for the vm through the `NyaHeapObject` type or the vm stack.
#[derive(Debug, Clone)]
pub enum NyaObject {
    HeapRef(NyaHeapObject),
    String(String),
    Number(f64),
    Table(HashMap<String, NyaHeapObject>),
    Nil,
}

impl NyaObject {
    /// marks children to not be freed by the gc
    pub fn mark_children(&mut self) {
        match self {
            NyaObject::String(_) | NyaObject::Number(_) | NyaObject::Nil => {}
            NyaObject::HeapRef(obj) => {
                if !obj.marked {
                    obj.marked = true;
                    obj.mark_children();
                }
            }
            NyaObject::Table(hash_map) => {
                for (_, obj) in hash_map.iter_mut() {
                    if !obj.marked {
                        obj.marked = true;
                        obj.mark_children();
                    }
                }
            }
        }
    }
}

/// This is the inner type to `NyaHeapObject` and tracks whether it has been marked for removal by
/// the gc.
#[derive(Debug, Clone)]
pub struct RawNyaHeapObject {
    pub inner: NyaObject,
    pub marked: bool,
}

impl Deref for RawNyaHeapObject {
    type Target = NyaObject;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for RawNyaHeapObject {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// This type is for the vm to track memory and should not be used outside of it.
/// Look at the garbage collector for how it is used.
///
/// # Safety
/// This type does not implment drop and will leak memory if `free(self)` is not called.
#[derive(Debug, Clone)]
pub struct NyaHeapObject {
    pub inner: *mut RawNyaHeapObject,
}

impl NyaHeapObject {
    /// Create a new object on the heap
    ///
    /// # Safety
    /// This structure does not implment drop so if it stops being tracked it will leak memory.
    /// This function is only meant to be used by the vm and it should never be needed
    /// outside of it.
    pub unsafe fn new(obj: NyaObject) -> Self {
        Self {
            inner: unsafe {
                let ptr = alloc(Layout::new::<RawNyaHeapObject>()) as *mut RawNyaHeapObject;
                if ptr.is_null() {
                    handle_alloc_error(Layout::new::<RawNyaHeapObject>())
                }
                *ptr = RawNyaHeapObject {
                    inner: obj,
                    marked: false,
                };
                ptr
            },
        }
    }

    /// Free this object from the heap
    ///
    /// # Safety
    /// Calling this while something holds a refrence too it still will cause an invalid access.
    /// This function should only be used by the garbage collector.
    pub unsafe fn free(self) {
        unsafe {
            dealloc(self.inner as *mut u8, Layout::new::<RawNyaHeapObject>());
        }
    }
}

impl Deref for NyaHeapObject {
    type Target = RawNyaHeapObject;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner }
    }
}

impl DerefMut for NyaHeapObject {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inner }
    }
}
