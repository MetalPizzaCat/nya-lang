use std::{
    cell::{Ref, RefMut},
    collections::HashMap,
};

use crate::{
    garbage_collect::{GcGuard, GcInnerGuard, GcInnerGuardMut, GcObject},
    state::NyaState,
};

/// This type holds a value for the vm through the vm stack.
#[derive(Debug, Clone, Copy)]
pub enum NyaPrimitiveType {
    HeapRef(GcObject),
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
                    raw.borrow_mut().mark_children();
                }
            }
            NyaPrimitiveType::Number(_) | NyaPrimitiveType::Int(_) | NyaPrimitiveType::Nil => {}
        }
    }

    pub fn into_hashable(self) -> Option<NyaHashableType> {
        match self {
            Self::Number(_) | Self::Nil => None,
            Self::Int(i) => Some(NyaHashableType::Int(i)),
            Self::HeapRef(heap_obj) => match &*heap_obj.borrow() {
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

// into trait

pub struct Nil;

pub trait IntoNyaType {
    fn into_nya_object(self, state: &mut NyaState) -> NyaPrimitiveType;
}

impl IntoNyaType for () {
    fn into_nya_object(self, state: &mut NyaState) -> NyaPrimitiveType {
        Nil.into_nya_object(state)
    }
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

impl IntoNyaType for GcGuard {
    fn into_nya_object(self, _state: &mut NyaState) -> NyaPrimitiveType {
        NyaPrimitiveType::HeapRef(*self)
    }
}

impl<T> IntoNyaType for GcInnerGuard<T> {
    fn into_nya_object(self, _state: &mut NyaState) -> NyaPrimitiveType {
        NyaPrimitiveType::HeapRef(*self.guard())
    }
}

impl<T> IntoNyaType for GcInnerGuardMut<T> {
    fn into_nya_object(self, _state: &mut NyaState) -> NyaPrimitiveType {
        NyaPrimitiveType::HeapRef(*self.guard())
    }
}

// from

pub trait FromNyaType
where
    Self: Sized,
{
    fn from_nya_object(object: NyaPrimitiveType) -> Option<Self>;
}

impl FromNyaType for f64 {
    fn from_nya_object(object: NyaPrimitiveType) -> Option<Self> {
        match object {
            NyaPrimitiveType::Number(n) => Some(n),
            _ => None,
        }
    }
}

impl FromNyaType for f32 {
    fn from_nya_object(object: NyaPrimitiveType) -> Option<Self> {
        match object {
            NyaPrimitiveType::Number(n) => Some(n as f32),
            _ => None,
        }
    }
}

impl FromNyaType for i64 {
    fn from_nya_object(object: NyaPrimitiveType) -> Option<Self> {
        match object {
            NyaPrimitiveType::Int(i) => Some(i),
            _ => None,
        }
    }
}

impl FromNyaType for u64 {
    fn from_nya_object(object: NyaPrimitiveType) -> Option<Self> {
        Some(i64::from_nya_object(object)? as u64)
    }
}

impl FromNyaType for usize {
    fn from_nya_object(object: NyaPrimitiveType) -> Option<Self> {
        Some(i64::from_nya_object(object)? as usize)
    }
}

impl FromNyaType for GcInnerGuard<String> {
    fn from_nya_object(object: NyaPrimitiveType) -> Option<Self> {
        if let NyaPrimitiveType::HeapRef(heap_ref) = object {
            heap_ref.create_guard().map_inner(|obj| {
                Ref::filter_map(obj.borrow(), |inner| match inner {
                    NyaHeapType::String(s) => Some(s),
                    _ => None,
                })
                .ok()
            })
        } else {
            None
        }
    }
}

impl FromNyaType for GcInnerGuardMut<String> {
    fn from_nya_object(object: NyaPrimitiveType) -> Option<Self> {
        if let NyaPrimitiveType::HeapRef(heap_ref) = object {
            heap_ref.create_guard().map_inner_mut(|obj| {
                RefMut::filter_map(obj.borrow_mut(), |inner| match inner {
                    NyaHeapType::String(s) => Some(s),
                    _ => None,
                })
                .ok()
            })
        } else {
            None
        }
    }
}
