use std::{
    alloc::{Layout, alloc, dealloc, handle_alloc_error},
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use crate::state::NyaState;

/// This type holds a value for the vm through the vm stack.
#[derive(Debug, Clone, Copy)]
pub enum NyaPrimitiveType {
    HeapRef(NyaHeapObject),
    Number(f64),
    Int(i64),
    Nil,
}

impl NyaPrimitiveType {
    /// marks references to the heap to not be freed by the gc
    pub fn mark_reference(&mut self) {
        match self {
            NyaPrimitiveType::HeapRef(obj) => {
                let raw = obj.get_raw_mut();
                if !raw.marked {
                    raw.marked = true;
                    raw.mark_children();
                }
            }
            NyaPrimitiveType::Number(_) | NyaPrimitiveType::Int(_) | NyaPrimitiveType::Nil => {}
        }
    }

    pub fn into_hashable(self) -> Option<NyaHashableType> {
        match self {
            Self::Number(_) | Self::Nil => None,
            Self::Int(i) => Some(NyaHashableType::Int(i)),
            Self::HeapRef(heap_obj) => match &*heap_obj {
                NyaHeapType::Table(_) => None,
                NyaHeapType::String(s) => Some(NyaHashableType::String(s.clone())),
            },
        }
    }
}

/// This type holds a value that is hashable
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum NyaHashableType {
    Int(i64),
    String(String),
}

/// This type holds a value for the vm through the heap
#[derive(Debug, Clone)]
pub enum NyaHeapType {
    Table(HashMap<NyaHashableType, NyaPrimitiveType>),
    String(String),
}

impl NyaHeapType {
    /// marks children to not be freed by the gc
    pub fn mark_children(&mut self) {
        match self {
            NyaHeapType::Table(hash_map) => {
                for (_, obj) in hash_map.iter_mut() {
                    obj.mark_reference();
                }
            }
            NyaHeapType::String(_) => {}
        }
    }
}

/// This is the inner type to `NyaHeapObject` and tracks whether it has been marked for removal by
/// the gc.
#[derive(Debug, Clone)]
pub struct RawNyaHeapObject {
    pub inner: NyaHeapType,
    pub marked: bool,
}

impl Deref for RawNyaHeapObject {
    type Target = NyaHeapType;
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
#[derive(Debug, Clone, Copy)]
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
    pub unsafe fn new(obj: NyaHeapType) -> Self {
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

    pub fn get_raw_mut(&self) -> &mut RawNyaHeapObject {
        unsafe { &mut *self.inner }
    }
}

impl Deref for NyaHeapObject {
    type Target = NyaHeapType;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(*self.inner) }
    }
}

impl DerefMut for NyaHeapObject {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(*self.inner) }
    }
}

// into trait

pub struct Nil;

pub trait IntoNyaType {
    fn into_nya_object(self, state: &mut NyaState) -> NyaPrimitiveType;
}

impl IntoNyaType for Nil {
    fn into_nya_object(self, _: &mut NyaState) -> NyaPrimitiveType {
        NyaPrimitiveType::Nil
    }
}

impl IntoNyaType for NyaPrimitiveType {
    fn into_nya_object(self, _: &mut NyaState) -> NyaPrimitiveType {
        self
    }
}

impl IntoNyaType for &str {
    fn into_nya_object(self, state: &mut NyaState) -> NyaPrimitiveType {
        let obj = state.alloc_heap_object(NyaHeapType::String(self.into()));
        NyaPrimitiveType::HeapRef(obj)
    }
}

impl IntoNyaType for String {
    fn into_nya_object(self, state: &mut NyaState) -> NyaPrimitiveType {
        let obj = state.alloc_heap_object(NyaHeapType::String(self));
        NyaPrimitiveType::HeapRef(obj)
    }
}

impl IntoNyaType for u8 {
    fn into_nya_object(self, _: &mut NyaState) -> NyaPrimitiveType {
        NyaPrimitiveType::Int(self as i64)
    }
}

impl IntoNyaType for u16 {
    fn into_nya_object(self, _: &mut NyaState) -> NyaPrimitiveType {
        NyaPrimitiveType::Int(self as i64)
    }
}

impl IntoNyaType for u32 {
    fn into_nya_object(self, _: &mut NyaState) -> NyaPrimitiveType {
        NyaPrimitiveType::Int(self as i64)
    }
}

impl IntoNyaType for u64 {
    fn into_nya_object(self, _: &mut NyaState) -> NyaPrimitiveType {
        NyaPrimitiveType::Int(self as i64)
    }
}

impl IntoNyaType for i8 {
    fn into_nya_object(self, _: &mut NyaState) -> NyaPrimitiveType {
        NyaPrimitiveType::Int(self as i64)
    }
}

impl IntoNyaType for i16 {
    fn into_nya_object(self, _: &mut NyaState) -> NyaPrimitiveType {
        NyaPrimitiveType::Int(self as i64)
    }
}

impl IntoNyaType for i32 {
    fn into_nya_object(self, _: &mut NyaState) -> NyaPrimitiveType {
        NyaPrimitiveType::Int(self as i64)
    }
}

impl IntoNyaType for i64 {
    fn into_nya_object(self, _: &mut NyaState) -> NyaPrimitiveType {
        NyaPrimitiveType::Int(self)
    }
}

impl IntoNyaType for f32 {
    fn into_nya_object(self, _: &mut NyaState) -> NyaPrimitiveType {
        NyaPrimitiveType::Number(self as f64)
    }
}

impl IntoNyaType for f64 {
    fn into_nya_object(self, _: &mut NyaState) -> NyaPrimitiveType {
        NyaPrimitiveType::Number(self)
    }
}

impl<T> IntoNyaType for Vec<T>
where
    T: IntoNyaType,
{
    fn into_nya_object(self, state: &mut NyaState) -> NyaPrimitiveType {
        let mut map = HashMap::new();
        for (i, v) in self.into_iter().enumerate() {
            if let Some(key) = NyaPrimitiveType::Int(i as i64).into_hashable() {
                let obj = v.into_nya_object(state);
                map.insert(key, obj);
            }
        }
        let heap_ref = state.alloc_heap_object(NyaHeapType::Table(map));
        NyaPrimitiveType::HeapRef(heap_ref)
    }
}

impl<T, const N: usize> IntoNyaType for [T; N]
where
    T: IntoNyaType,
{
    fn into_nya_object(self, state: &mut NyaState) -> NyaPrimitiveType {
        let mut map = HashMap::new();
        for (i, v) in self.into_iter().enumerate() {
            if let Some(key) = NyaPrimitiveType::Int(i as i64).into_hashable() {
                let obj = v.into_nya_object(state);
                map.insert(key, obj);
            }
        }
        let heap_ref = state.alloc_heap_object(NyaHeapType::Table(map));
        NyaPrimitiveType::HeapRef(heap_ref)
    }
}

impl<K, V> IntoNyaType for HashMap<K, V>
where
    K: IntoNyaType,
    V: IntoNyaType,
{
    fn into_nya_object(self, state: &mut NyaState) -> NyaPrimitiveType {
        let mut map = HashMap::new();
        for (k, v) in self {
            if let Some(key) = k.into_nya_object(state).into_hashable() {
                map.insert(key, v.into_nya_object(state));
            }
        }
        let heap_ref = state.alloc_heap_object(NyaHeapType::Table(map));
        NyaPrimitiveType::HeapRef(heap_ref)
    }
}

// from

#[derive(Debug)]
pub struct NotCorrectTypeError;

impl TryFrom<NyaPrimitiveType> for f64 {
    type Error = NotCorrectTypeError;
    fn try_from(value: NyaPrimitiveType) -> Result<Self, Self::Error> {
        match value {
            NyaPrimitiveType::Number(n) => Ok(n),
            _ => Err(NotCorrectTypeError),
        }
    }
}

impl TryFrom<NyaPrimitiveType> for f32 {
    type Error = NotCorrectTypeError;
    fn try_from(value: NyaPrimitiveType) -> Result<Self, Self::Error> {
        match value {
            NyaPrimitiveType::Number(n) => Ok(n as f32),
            _ => Err(NotCorrectTypeError),
        }
    }
}

impl TryFrom<NyaPrimitiveType> for i64 {
    type Error = NotCorrectTypeError;
    fn try_from(value: NyaPrimitiveType) -> Result<Self, Self::Error> {
        match value {
            NyaPrimitiveType::Int(i) => Ok(i),
            _ => Err(NotCorrectTypeError),
        }
    }
}

impl TryFrom<NyaPrimitiveType> for u64 {
    type Error = NotCorrectTypeError;
    fn try_from(value: NyaPrimitiveType) -> Result<Self, Self::Error> {
        Ok(i64::try_from(value)? as u64)
    }
}

impl TryFrom<NyaPrimitiveType> for usize {
    type Error = NotCorrectTypeError;
    fn try_from(value: NyaPrimitiveType) -> Result<Self, Self::Error> {
        Ok(i64::try_from(value)? as usize)
    }
}

impl TryFrom<NyaPrimitiveType> for String {
    type Error = NotCorrectTypeError;
    fn try_from(value: NyaPrimitiveType) -> Result<Self, Self::Error> {
        if let NyaPrimitiveType::HeapRef(heap_ref) = value {
            match (*heap_ref).clone() {
                NyaHeapType::String(s) => Ok(s),
                _ => Err(NotCorrectTypeError),
            }
        } else {
            Err(NotCorrectTypeError)
        }
    }
}
