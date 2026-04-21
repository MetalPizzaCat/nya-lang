use std::{
    alloc::{Layout, alloc, dealloc, handle_alloc_error},
    cell::{Ref, RefCell, RefMut},
    collections::HashSet,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use crate::object::{NyaHeapType, NyaPrimitiveType};

/// This is the inner type to `NyaHeapObject` and tracks whether it has been marked for removal by
/// the gc.
#[derive(Debug, Clone)]
pub struct RawGcObject {
    pub inner: RefCell<NyaHeapType>,
    pub marked: bool,
}

impl Deref for RawGcObject {
    type Target = RefCell<NyaHeapType>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for RawGcObject {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// This type is for the vm to track memory and should not be used outside of it.
/// Look at the garbage collector for how it is used.
///
/// # Safety
/// This type does not implment drop and will leak memory if `free(self)` is not called.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct GcObject {
    pub gc: *mut GarbageCollector,
    pub inner: *mut RawGcObject,
}

impl GcObject {
    /// Create a new object on the heap
    ///
    /// # Safety
    /// This structure does not implment drop so if it stops being tracked it will leak memory.
    /// This function is only meant to be used by the vm and it should never be needed
    /// outside of it.
    pub unsafe fn new(gc: &mut GarbageCollector, obj: NyaHeapType) -> Self {
        Self {
            gc,
            inner: unsafe {
                let ptr = alloc(Layout::new::<RawGcObject>()) as *mut RawGcObject;
                if ptr.is_null() {
                    handle_alloc_error(Layout::new::<RawGcObject>())
                }
                ptr.write(RawGcObject {
                    inner: RefCell::new(obj),
                    marked: false,
                });
                ptr
            },
        }
    }

    pub fn create_guard(self) -> GcGuard {
        unsafe { (*self.gc).create_guard(self) }
    }

    /// Free this object from the heap
    ///
    /// # Safety
    /// Calling this while something holds a refrence too it still will cause an invalid access.
    /// This function should only be used by the garbage collector.
    pub unsafe fn free(self) {
        unsafe {
            dealloc(self.inner as *mut u8, Layout::new::<RawGcObject>());
        }
    }

    pub fn get_raw_mut(&self) -> &mut RawGcObject {
        unsafe { &mut *self.inner }
    }
}

impl Deref for GcObject {
    type Target = RefCell<NyaHeapType>;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner }
    }
}

impl DerefMut for GcObject {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inner }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GcGuard {
    inner: Rc<GcObject>,
}

impl Deref for GcGuard {
    type Target = GcObject;
    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl GcGuard {
    pub fn map_inner<T, F>(self, f: F) -> Option<GcInnerGuard<T>>
    where
        F: FnOnce(&RefCell<NyaHeapType>) -> Option<Ref<'_, T>>,
    {
        let refcell = &**self.inner;

        let value_ref = f(refcell)?;

        unsafe {
            let extended_ref = std::mem::transmute::<Ref<'_, T>, Ref<'static, T>>(value_ref);

            Some(GcInnerGuard {
                _gc_guard: self.inner.clone(),
                value: extended_ref,
            })
        }
    }

    pub fn map_inner_mut<T, F>(self, f: F) -> Option<GcInnerGuardMut<T>>
    where
        F: FnOnce(&RefCell<NyaHeapType>) -> Option<RefMut<'_, T>>,
    {
        let refcell = &**self.inner;

        let value_ref = f(refcell)?;

        unsafe {
            let extended_ref = std::mem::transmute::<RefMut<'_, T>, RefMut<'static, T>>(value_ref);

            Some(GcInnerGuardMut {
                _gc_guard: self.inner.clone(),
                value: extended_ref,
            })
        }
    }
}

pub struct GcInnerGuard<T: 'static> {
    _gc_guard: Rc<GcObject>,
    value: Ref<'static, T>,
}

impl<T: 'static> GcInnerGuard<T> {
    pub fn guard(self) -> GcGuard {
        GcGuard {
            inner: self._gc_guard,
        }
    }
}

impl<T> Deref for GcInnerGuard<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

pub struct GcInnerGuardMut<T: 'static> {
    _gc_guard: Rc<GcObject>,
    value: RefMut<'static, T>,
}

impl<T: 'static> GcInnerGuardMut<T> {
    pub fn guard(self) -> GcGuard {
        GcGuard {
            inner: self._gc_guard,
        }
    }
}

impl<T> Deref for GcInnerGuardMut<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for GcInnerGuardMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

pub struct GarbageCollector {
    heap: Vec<GcObject>,
    guards: HashSet<GcGuard>,
}

impl GarbageCollector {
    pub fn new() -> Self {
        Self {
            heap: Vec::new(),
            guards: HashSet::new(),
        }
    }

    pub fn alloc(&mut self, obj: NyaHeapType) -> GcObject {
        let heap_obj = unsafe { GcObject::new(self, obj) };
        self.heap.push(heap_obj);
        heap_obj
    }

    pub fn mark_slice(&mut self, objects: &mut [NyaPrimitiveType]) {
        for obj in objects {
            if let NyaPrimitiveType::HeapRef(obj) = obj {
                let raw = obj.get_raw_mut();
                raw.marked = true;
                if let Ok(mut value) = raw.try_borrow_mut() {
                    value.mark_children()
                }
            }
        }
    }

    pub fn mark(&mut self, object: &mut NyaPrimitiveType) {
        if let NyaPrimitiveType::HeapRef(obj) = object {
            let raw = obj.get_raw_mut();
            raw.marked = true;
            if let Ok(mut value) = raw.try_borrow_mut() {
                value.mark_children()
            }
        }
    }

    pub fn create_guard(&mut self, object: GcObject) -> GcGuard {
        if let Some(guard) = self.guards.iter().find(|g| *g.inner == object) {
            guard.clone()
        } else {
            let guard = GcGuard {
                inner: Rc::new(object),
            };
            self.guards.insert(guard.clone());
            guard
        }
    }

    pub unsafe fn sweep(&mut self) {
        self.guards.retain(|g| Rc::strong_count(&g.inner) > 1);

        for g in self.guards.iter() {
            let raw = g.get_raw_mut();
            raw.marked = true;
            unsafe {
                (*g.as_ptr()).mark_children();
            }
        }

        for i in (0..self.heap.len()).rev() {
            if !self.heap[i].get_raw_mut().marked {
                let obj = self.heap.swap_remove(i);
                println!("freed {:?}", *obj);
                unsafe {
                    obj.free();
                }
            }
        }

        for obj in &mut self.heap {
            let raw = obj.get_raw_mut();
            raw.marked = false;
        }
    }
}

impl Drop for GarbageCollector {
    fn drop(&mut self) {
        for obj in &self.heap {
            unsafe {
                obj.free();
            }
        }
    }
}
