use std::{ops::Not, sync::Arc};

use crate::{
    object::{FromNyaType, NyaPrimitiveType},
    state::NyaState,
};

#[derive(Clone)]
pub struct RustClosure {
    func: Arc<dyn CallableRustClosure>,
}

impl RustClosure {
    pub fn call(&self, state: &mut NyaState) -> Option<()> {
        self.func.call(state)
    }
}

pub trait FromGenericCallable<F, R0, Args>
where
    F: IntoFunctionGenericType<R0, Args> + 'static,
{
    fn from_callable(func: F) -> Self;
}

pub trait CallableRustClosure {
    fn call(&self, state: &mut NyaState) -> Option<()>;
}

pub struct FunctionGenericType<F, R, Args> {
    pub inner: F,
    _args_marker: std::marker::PhantomData<Args>,
    _return_marker: std::marker::PhantomData<R>,
}

pub trait IntoFunctionGenericType<R, Args>
where
    Self: Sized,
{
    fn into_function_generic(self) -> FunctionGenericType<Self, R, Args>;
}

// impl<F, R0, T1> FromGenericCallable<F, R0, (T1)> for RustClosure
// where
//     R0: crate::object::IntoNyaType + 'static,
//     T1: FromNyaType + 'static,
//     F: IntoFunctionGenericType<R0, (T1)> + 'static,
//     FunctionGenericType<F, R0, (T1)>: CallableRustClosure,
// {
//     fn from_callable(func: F) -> Self {
//         Self {
//             func: Arc::new(func.into_function_generic()),
//         }
//     }
// }

macro_rules! impl_callable_for_fn {
    ($($T:ident),*) => {

        impl<F, R0, $($T),*> FromGenericCallable<F, R0, ($($T,)*)> for RustClosure
        where
            R0: crate::object::IntoNyaType + 'static,
            $($T: $crate::object::FromNyaType + 'static),*,
            F: IntoFunctionGenericType<R0, ($($T,)*)> + 'static,
            FunctionGenericType<F, R0, ($($T,)*)>: CallableRustClosure,
        {
            fn from_callable(func: F) -> Self {
                Self {
                    func: Arc::new(func.into_function_generic()),
                }
            }
        }

        impl<F, R0, $($T),*> IntoFunctionGenericType<R0, ($($T,)*)> for F
        where
            F: Fn($($T),*) -> R0,
        {
            fn into_function_generic(self) -> FunctionGenericType<Self, R0, ($($T,)*)> { FunctionGenericType {
                    inner: self,
                    _args_marker: std::marker::PhantomData::<($($T,)*)>,
                    _return_marker: std::marker::PhantomData::<R0>,
                }
            }
        }

        impl<F, R0, $($T),*> CallableRustClosure for FunctionGenericType<F, R0, ($($T,)*)>
        where
            F: Fn($($T),*) -> R0,
            R0: $crate::object::IntoNyaType,
            $($T: $crate::object::FromNyaType),*
        {

            #[allow(non_snake_case)]
            fn call(&self, state: &mut NyaState) -> Option<()> {
                let arg_count = [$(stringify!($T)),*].len() as isize;
                let mut index = 0;

                $(
                    index += 1;
                    let $T = state.get_stack(index - 1 - arg_count)?;
                )*

                let r0 = (self.inner)($($T),*);
                state.push_value(r0);
                Some(())
            }
        }
    };
}

macro_rules! generate_all_callables {
    () => {
        // stop
    };
    ($head:ident $(, $tail:ident)*) => {
        impl_callable_for_fn!($head $(, $tail)*);
        generate_all_callables!($($tail),*);
    };
}

generate_all_callables!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);

// impl<T> CallableRustClosure for fn(T)
// where
//     T: FromNyaType,
// {
//     fn call(&self, state: &mut NyaState) -> Option<()> {
//         let arg1 = state.get_stack(-1)?;
//         (*self)(arg1);
//         Some(())
//     }
// }
