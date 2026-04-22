use std::{
    alloc::{Layout, alloc, dealloc, handle_alloc_error},
    cell::{Ref, RefCell, RefMut},
    ops::{Deref, DerefMut},
    rc::Rc,
};

use crate::object::{NyaHeapObject, NyaPrimitiveObject};

/// This is the inner type to [`GcObject`] and tracks whether it has been marked for removal by
/// the gc and owns the [`NyaHeapObject`].
#[derive(Debug, Clone)]
struct RawGcObject {
    pub inner: RefCell<NyaHeapObject>,
    pub marked: bool,
}

impl Deref for RawGcObject {
    type Target = RefCell<NyaHeapObject>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for RawGcObject {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// [`GcObject`] holds a heap allocated reference to a heap object.
/// This type should only be created by the [`GarbageCollector`] and should never leave the [`crate::state::NyaState`] without a guard.
///
/// # Safety
/// If a [`GcObject`] is not marked by [`crate::state::NyaState`] before a [`GarbageCollector::sweep()`] call it will get freed.
/// This type does not implment drop and will leak memory if [`Self::free()`] is not called.
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
    unsafe fn new(gc: &mut GarbageCollector, obj: NyaHeapObject) -> Self {
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

    /// Creates a GcHeapGuard from a GcObject allowing it to be safely taken out of the VM.
    /// Note that having any GcHeapGuard exist will stop the GC from freeing any memory.
    ///
    /// See [`GcHeapGuard`] for examples
    pub fn create_guard(self) -> GcHeapGuard {
        unsafe { (*self.gc).create_guard(self) }
    }

    /// Free this object from the heap
    ///
    /// # Safety
    /// Calling this while something holds a refrence too it still will cause a use after free.
    /// This function should only be used by the garbage collector.
    unsafe fn free(self) {
        unsafe {
            dealloc(self.inner as *mut u8, Layout::new::<RawGcObject>());
        }
    }

    /// Mark this object as used so it does not get freed.
    ///
    /// # Safety
    /// This function breaks the borrow rules so that the [`GarbageCollector`] can always be marked
    ///
    /// # Examples
    /// ```rust,ignore
    /// if !object.is_marked() {
    ///     object.mark();
    ///     object.borrow_mut().mark_children();
    /// }
    /// ```
    pub fn mark(&self) {
        unsafe {
            (*self.inner).marked = true;
        }
    }

    /// returns true if this object has been marked as used.
    ///
    /// # Examples
    /// ```rust,ignore
    /// if !object.is_marked() {
    ///     object.mark();
    ///     object.borrow_mut().mark_children();
    /// }
    /// ```
    pub fn is_marked(&self) -> bool {
        unsafe { (*self.inner).marked }
    }
}

impl Deref for GcObject {
    type Target = RefCell<NyaHeapObject>;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner }
    }
}

impl DerefMut for GcObject {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inner }
    }
}

/// GcHeapGuard lets you hold a safe reference to [`GcObject`] without it being freed.
/// Note that it does this by stopping the [`GarbageCollector`] from freeing any memory.
/// This this useful for moving objects out of the [`crate::state::NyaState`].
///
/// # Examples
/// ```rust,ignore
/// let safe_ref = object.create_guard();
/// match *safe_ref {
///     ...
/// }
/// ```
#[derive(Debug, Clone)]
pub struct GcHeapGuard {
    object: GcObject,
    guard: Rc<()>,
}

impl Deref for GcHeapGuard {
    type Target = GcObject;
    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl GcHeapGuard {
    /// Map the value inside the [`NyaHeapObject`] enum to a [`GcInnerGuard`] for easier direct access by reference.
    ///
    /// # Examples
    /// ```rust,ignore
    /// if let NyaPrimitiveObject::HeapRef(heap_ref) = object {
    ///     heap_ref.create_guard().map_inner(|obj| {
    ///         Ref::filter_map(obj.borrow(), |inner| match inner {
    ///             NyaHeapObject::String(s) => Some(s),
    ///             _ => None,
    ///         })
    ///         .ok()
    ///     })
    /// } else {
    ///     None
    /// }
    /// ```
    pub fn map_inner<T, F>(self, f: F) -> Option<GcInnerGuard<T>>
    where
        F: FnOnce(&RefCell<NyaHeapObject>) -> Option<Ref<'_, T>>,
    {
        let refcell = &*self.object;

        let value_ref = f(refcell)?;

        unsafe {
            let extended_ref = std::mem::transmute::<Ref<'_, T>, Ref<'static, T>>(value_ref);

            Some(GcInnerGuard {
                guard: self.guard,
                object: self.object,
                value: extended_ref,
            })
        }
    }

    /// Map the value inside the [`NyaHeapObject`] enum to a [`GcInnerGuardMut`] for easier direct mutable access by reference.
    ///
    /// # Examples
    /// ```rust,ignore
    /// if let NyaPrimitiveObject::HeapRef(heap_ref) = object {
    ///     heap_ref.create_guard().map_inner_mut(|obj| {
    ///         RefMut::filter_map(obj.borrow_mut(), |inner| match inner {
    ///             NyaHeapObject::String(s) => Some(s),
    ///             _ => None,
    ///         })
    ///         .ok()
    ///     })
    /// } else {
    ///     None
    /// }
    /// ```
    pub fn map_inner_mut<T, F>(self, f: F) -> Option<GcInnerGuardMut<T>>
    where
        F: FnOnce(&RefCell<NyaHeapObject>) -> Option<RefMut<'_, T>>,
    {
        let refcell = &*self.object;

        let value_ref = f(refcell)?;

        unsafe {
            let extended_ref = std::mem::transmute::<RefMut<'_, T>, RefMut<'static, T>>(value_ref);

            Some(GcInnerGuardMut {
                guard: self.guard,
                object: self.object,
                value: extended_ref,
            })
        }
    }
}

/// A type created from [`GcHeapGuard`] to get direct access to the type inside the [`NyaHeapObject`] by reference.
/// Created by [`GcHeapGuard::map_inner()`]
///
/// # Examples
/// ```rust,ignore
/// if let Some(let) inner_guard = GcInnerGuard::<String>::from_nya_object(object) {
///     println!("{}", *inner_guard);
/// }
/// ```
#[derive(Debug)]
pub struct GcInnerGuard<T: 'static> {
    guard: Rc<()>,
    object: GcObject,
    value: Ref<'static, T>,
}

impl<T: 'static> GcInnerGuard<T> {
    /// Turn self back into a generic [`GcHeapGuard`].
    pub fn guard(self) -> GcHeapGuard {
        GcHeapGuard {
            guard: self.guard,
            object: self.object,
        }
    }
}

impl<T> Deref for GcInnerGuard<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// A type created from [`GcHeapGuard`] to get direct mutable access to the type inside the [`NyaHeapObject`] by reference.
/// Created by [`GcHeapGuard::map_inner_mut()`]
///
/// # Examples
/// ```rust,ignore
/// if let Some(let) inner_guard_mut = GcInnerGuardMut::<String>::from_nya_object(object) {
///     inner_guard_mut.push_str(":3");
///     println!("{}", *inner_guard);
/// }
/// ```
#[derive(Debug)]
pub struct GcInnerGuardMut<T: 'static> {
    guard: Rc<()>,
    object: GcObject,
    value: RefMut<'static, T>,
}

impl<T: 'static> GcInnerGuardMut<T> {
    /// Turn self back into a generic [`GcHeapGuard`].
    pub fn guard(self) -> GcHeapGuard {
        GcHeapGuard {
            guard: self.guard,
            object: self.object,
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

/// The garbage collector for [`crate::state::NyaState`].
///
/// # Saftey
/// Any [`GcObject`] created by the [`GarbageCollector`] should not be used outside the
/// [`crate::state::NyaState`].
pub struct GarbageCollector {
    heap: Vec<GcObject>,
    guard: Rc<()>,
}

impl GarbageCollector {
    /// Create a new garbage collector.
    pub fn new() -> Self {
        Self {
            heap: Vec::new(),
            guard: Rc::new(()),
        }
    }

    /// Create a new object in the garbage collector returning a [`GcObject`] reference to it.
    pub fn alloc(&mut self, obj: NyaHeapObject) -> GcObject {
        let heap_obj = unsafe { GcObject::new(self, obj) };
        self.heap.push(heap_obj);
        heap_obj
    }

    /// Mark a slice of objects that are in use to not be freed.
    /// Always make sure that all objects in use are marked or inside a [`GcHeapGuard`].
    ///
    /// # Examples
    /// ```rust,ignore
    /// let slice = [0.into_nya_object(&mut state), ":3".into_nya_object(&mut state)];
    /// garbage_collector.mark_slice(&mut slice);
    /// garbage_collector.sweep();
    /// ```
    pub fn mark_slice(&mut self, objects: &mut [NyaPrimitiveObject]) {
        for obj in objects {
            if let NyaPrimitiveObject::HeapRef(obj) = obj {
                let raw = unsafe { &mut (*obj.inner) };
                raw.marked = true;
                if let Ok(mut value) = raw.try_borrow_mut() {
                    value.mark_children()
                }
            }
        }
    }

    /// Mark a single object as in use to not be freed.
    /// Always make sure that all objects in use are marked or inside a [`GcHeapGuard`].
    ///
    /// # Examples
    /// ```rust,ignore
    /// let object = 0.into_nya_object(&mut state);
    /// garbage_collector.mark(&mut object);
    /// garbage_collector.sweep();
    /// ```
    pub fn mark(&mut self, object: &mut NyaPrimitiveObject) {
        if let NyaPrimitiveObject::HeapRef(obj) = object {
            let raw = unsafe { &mut (*obj.inner) };
            raw.marked = true;
            if let Ok(mut value) = raw.try_borrow_mut() {
                value.mark_children()
            }
        }
    }

    /// Create a [`GcHeapGuard`] from a [`GcObject`]
    /// See [`GcHeapGuard`] for more.
    pub fn create_guard(&mut self, object: GcObject) -> GcHeapGuard {
        GcHeapGuard {
            guard: self.guard.clone(),
            object,
        }
    }

    /// Free all unused objects unless a [`GcHeapGuard`] exists. If a [`GcHeapGuard`] exists nothing will
    /// be freed.
    ///
    /// # Safety
    /// This function will free anything unmarked so everything in use must be marked first or be
    /// behind a [`GcHeapGuard`].
    pub unsafe fn sweep(&mut self) {
        if Rc::strong_count(&self.guard) > 1 {
            return;
        }

        for i in (0..self.heap.len()).rev() {
            if !self.heap[i].is_marked() {
                let obj = self.heap.swap_remove(i);
                println!("freed {:?}", *obj);
                unsafe {
                    obj.free();
                }
            }
        }

        for obj in &mut self.heap {
            let raw = unsafe { &mut (*obj.inner) };
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
