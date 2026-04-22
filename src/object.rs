use std::{
    cell::{Ref, RefMut},
    collections::HashMap,
};

use crate::{
    garbage_collector::{GcHeapGuard, GcInnerGuard, GcInnerGuardMut, GcObject},
    state::NyaState,
};

/// This type holds a value on the stack or likewise for [`NyaState`] and can also hold a reference
/// to a heap object.
#[derive(Debug, Clone, Copy)]
pub enum NyaPrimitiveObject {
    HeapRef(GcObject),
    Number(f64),
    Int(i64),
    Nil,
}

impl NyaPrimitiveObject {
    /// marks heap references to the heap to not be freed by the
    /// [`crate::garbage_collector::GarbageCollector`].
    pub fn mark_reference(&mut self) {
        match self {
            NyaPrimitiveObject::HeapRef(obj) => {
                if !obj.is_marked() {
                    obj.mark();
                    obj.borrow_mut().mark_children();
                }
            }
            NyaPrimitiveObject::Number(_)
            | NyaPrimitiveObject::Int(_)
            | NyaPrimitiveObject::Nil => {}
        }
    }

    /// Consumes [`self`] to turn it into a hashable type to be used in a table.
    pub fn into_hashable(self) -> Option<NyaHashableType> {
        match self {
            Self::Number(_) | Self::Nil => None,
            Self::Int(i) => Some(NyaHashableType::Int(i)),
            Self::HeapRef(heap_obj) => match &*heap_obj.borrow() {
                NyaHeapObject::Table(_) => None,
                NyaHeapObject::String(s) => Some(NyaHashableType::String(s.clone())),
            },
        }
    }
}

/// This type holds a value that is hashable for use in a table.
/// It is made by [`NyaPrimitiveObject::into_hashable()`]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum NyaHashableType {
    Int(i64),
    String(String),
}

/// This type holds a value for the [`NyaState`] that should live on the heap through the
/// [`crate::garbage_collector::GarbageCollector`].
#[derive(Debug, Clone)]
pub enum NyaHeapObject {
    Table(HashMap<NyaHashableType, NyaPrimitiveObject>),
    String(String),
}

impl NyaHeapObject {
    /// marks children to not be freed by the [`crate::garbage_collector::GarbageCollector`].
    pub fn mark_children(&mut self) {
        match self {
            NyaHeapObject::Table(hash_map) => {
                for (_, obj) in hash_map.iter_mut() {
                    obj.mark_reference();
                }
            }
            NyaHeapObject::String(_) => {}
        }
    }
}

/// `Nil` type as a structure that implments [`IntoNyaObject`] and [`FromNyaObject`].
pub struct Nil;

impl IntoNyaObject for Nil {
    fn into_nya_object(self, _: &mut NyaState) -> NyaPrimitiveObject {
        NyaPrimitiveObject::Nil
    }
}

impl FromNyaObject for Nil {
    fn from_nya_object(object: NyaPrimitiveObject) -> Option<Self> {
        match object {
            NyaPrimitiveObject::Nil => Some(Nil),
            _ => None,
        }
    }
}

// into trait

/// Consumes [`self`] to turn it into a [`NyaPrimitiveObject`] and may make an heap allocation
/// through the [`NyaState`].
pub trait IntoNyaObject {
    fn into_nya_object(self, state: &mut NyaState) -> NyaPrimitiveObject;
}

impl IntoNyaObject for NyaPrimitiveObject {
    fn into_nya_object(self, _: &mut NyaState) -> NyaPrimitiveObject {
        self
    }
}

impl IntoNyaObject for &str {
    fn into_nya_object(self, state: &mut NyaState) -> NyaPrimitiveObject {
        let obj = state.alloc_heap_object(NyaHeapObject::String(self.into()));
        NyaPrimitiveObject::HeapRef(obj)
    }
}

impl IntoNyaObject for String {
    fn into_nya_object(self, state: &mut NyaState) -> NyaPrimitiveObject {
        let obj = state.alloc_heap_object(NyaHeapObject::String(self));
        NyaPrimitiveObject::HeapRef(obj)
    }
}

impl IntoNyaObject for u8 {
    fn into_nya_object(self, _: &mut NyaState) -> NyaPrimitiveObject {
        NyaPrimitiveObject::Int(self as i64)
    }
}

impl IntoNyaObject for u16 {
    fn into_nya_object(self, _: &mut NyaState) -> NyaPrimitiveObject {
        NyaPrimitiveObject::Int(self as i64)
    }
}

impl IntoNyaObject for u32 {
    fn into_nya_object(self, _: &mut NyaState) -> NyaPrimitiveObject {
        NyaPrimitiveObject::Int(self as i64)
    }
}

impl IntoNyaObject for u64 {
    fn into_nya_object(self, _: &mut NyaState) -> NyaPrimitiveObject {
        NyaPrimitiveObject::Int(self as i64)
    }
}

impl IntoNyaObject for i8 {
    fn into_nya_object(self, _: &mut NyaState) -> NyaPrimitiveObject {
        NyaPrimitiveObject::Int(self as i64)
    }
}

impl IntoNyaObject for i16 {
    fn into_nya_object(self, _: &mut NyaState) -> NyaPrimitiveObject {
        NyaPrimitiveObject::Int(self as i64)
    }
}

impl IntoNyaObject for i32 {
    fn into_nya_object(self, _: &mut NyaState) -> NyaPrimitiveObject {
        NyaPrimitiveObject::Int(self as i64)
    }
}

impl IntoNyaObject for i64 {
    fn into_nya_object(self, _: &mut NyaState) -> NyaPrimitiveObject {
        NyaPrimitiveObject::Int(self)
    }
}

impl IntoNyaObject for f32 {
    fn into_nya_object(self, _: &mut NyaState) -> NyaPrimitiveObject {
        NyaPrimitiveObject::Number(self as f64)
    }
}

impl IntoNyaObject for f64 {
    fn into_nya_object(self, _: &mut NyaState) -> NyaPrimitiveObject {
        NyaPrimitiveObject::Number(self)
    }
}

impl<T> IntoNyaObject for Vec<T>
where
    T: IntoNyaObject,
{
    fn into_nya_object(self, state: &mut NyaState) -> NyaPrimitiveObject {
        let mut map = HashMap::new();
        for (i, v) in self.into_iter().enumerate() {
            if let Some(key) = NyaPrimitiveObject::Int(i as i64).into_hashable() {
                let obj = v.into_nya_object(state);
                map.insert(key, obj);
            }
        }
        let heap_ref = state.alloc_heap_object(NyaHeapObject::Table(map));
        NyaPrimitiveObject::HeapRef(heap_ref)
    }
}

impl<T, const N: usize> IntoNyaObject for [T; N]
where
    T: IntoNyaObject,
{
    fn into_nya_object(self, state: &mut NyaState) -> NyaPrimitiveObject {
        let mut map = HashMap::new();
        for (i, v) in self.into_iter().enumerate() {
            if let Some(key) = NyaPrimitiveObject::Int(i as i64).into_hashable() {
                let obj = v.into_nya_object(state);
                map.insert(key, obj);
            }
        }
        let heap_ref = state.alloc_heap_object(NyaHeapObject::Table(map));
        NyaPrimitiveObject::HeapRef(heap_ref)
    }
}

impl<K, V> IntoNyaObject for HashMap<K, V>
where
    K: IntoNyaObject,
    V: IntoNyaObject,
{
    fn into_nya_object(self, state: &mut NyaState) -> NyaPrimitiveObject {
        let mut map = HashMap::new();
        for (k, v) in self {
            if let Some(key) = k.into_nya_object(state).into_hashable() {
                map.insert(key, v.into_nya_object(state));
            }
        }
        let heap_ref = state.alloc_heap_object(NyaHeapObject::Table(map));
        NyaPrimitiveObject::HeapRef(heap_ref)
    }
}

impl IntoNyaObject for GcHeapGuard {
    fn into_nya_object(self, _state: &mut NyaState) -> NyaPrimitiveObject {
        NyaPrimitiveObject::HeapRef(*self)
    }
}

impl<T> IntoNyaObject for GcInnerGuard<T> {
    fn into_nya_object(self, _state: &mut NyaState) -> NyaPrimitiveObject {
        NyaPrimitiveObject::HeapRef(*self.guard())
    }
}

impl<T> IntoNyaObject for GcInnerGuardMut<T> {
    fn into_nya_object(self, _state: &mut NyaState) -> NyaPrimitiveObject {
        NyaPrimitiveObject::HeapRef(*self.guard())
    }
}

// from

/// Consumes [`NyaPrimitiveObject`] to turn it into [`self`] which may be done through a heap guard.
pub trait FromNyaObject
where
    Self: Sized,
{
    fn from_nya_object(object: NyaPrimitiveObject) -> Option<Self>;
}

impl FromNyaObject for () {
    fn from_nya_object(object: NyaPrimitiveObject) -> Option<Self> {
        match object {
            NyaPrimitiveObject::Nil => Some(()),
            _ => None,
        }
    }
}

impl FromNyaObject for f64 {
    fn from_nya_object(object: NyaPrimitiveObject) -> Option<Self> {
        match object {
            NyaPrimitiveObject::Number(n) => Some(n),
            _ => None,
        }
    }
}

impl FromNyaObject for f32 {
    fn from_nya_object(object: NyaPrimitiveObject) -> Option<Self> {
        match object {
            NyaPrimitiveObject::Number(n) => Some(n as f32),
            _ => None,
        }
    }
}

impl FromNyaObject for i64 {
    fn from_nya_object(object: NyaPrimitiveObject) -> Option<Self> {
        match object {
            NyaPrimitiveObject::Int(i) => Some(i),
            _ => None,
        }
    }
}

impl FromNyaObject for u64 {
    fn from_nya_object(object: NyaPrimitiveObject) -> Option<Self> {
        Some(i64::from_nya_object(object)? as u64)
    }
}

impl FromNyaObject for usize {
    fn from_nya_object(object: NyaPrimitiveObject) -> Option<Self> {
        Some(i64::from_nya_object(object)? as usize)
    }
}

impl FromNyaObject for GcInnerGuard<String> {
    fn from_nya_object(object: NyaPrimitiveObject) -> Option<Self> {
        if let NyaPrimitiveObject::HeapRef(heap_ref) = object {
            heap_ref.create_guard().map_inner(|obj| {
                Ref::filter_map(obj.borrow(), |inner| match inner {
                    NyaHeapObject::String(s) => Some(s),
                    _ => None,
                })
                .ok()
            })
        } else {
            None
        }
    }
}

impl FromNyaObject for GcInnerGuardMut<String> {
    fn from_nya_object(object: NyaPrimitiveObject) -> Option<Self> {
        if let NyaPrimitiveObject::HeapRef(heap_ref) = object {
            heap_ref.create_guard().map_inner_mut(|obj| {
                RefMut::filter_map(obj.borrow_mut(), |inner| match inner {
                    NyaHeapObject::String(s) => Some(s),
                    _ => None,
                })
                .ok()
            })
        } else {
            None
        }
    }
}
