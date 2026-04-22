use std::sync::Arc;

use crate::state::NyaState;

// We create a [`FunctionGenericType`] from any function.
// `FunctionGenericType<T, R, Args>` implements [`CallableRustClosure`].
// [`RustFunction`] accepts any type that implements [`IntoFunctionGenericType`]

// RustFunction is the only type in this module you should have to worry about... probably.

/// This type holds a callable rust function and is created with the [`FromGenericCallable`] trait.
///
/// # Examples
/// ```rust,ignore
/// let mut ns = NyaState::new();
/// ns.create_stack_frame();
/// ns.push_value(10);
/// ns.push_value(5);
///
/// let func = RustFunction::from_callable(|a: i64, b: i64| -> i64 { a + b });
/// func.call(&mut ns);
///
/// let value: i64 = ns.get_value(-1);
/// assert_e1!(value, 15);
/// ```
#[derive(Clone)]
pub struct RustFunction {
    func: Arc<dyn CallableRustClosure>,
}

impl RustFunction {
    /// Call the function poping its paremeters off the stack and push its result onto it.
    pub fn call(&self, state: &mut NyaState) -> Option<()> {
        self.func.call(state)
    }
}

/// Turn a generic function into [`self`].
pub trait FromGenericCallable<F, R0, Args>
where
    F: IntoFunctionGenericType<R0, Args> + 'static,
{
    fn from_callable(func: F) -> Self;
}

/// Generic trait for calling an object.
pub trait CallableRustClosure {
    fn call(&self, state: &mut NyaState) -> Option<()>;
}

/// Represents any rust function.
pub struct FunctionGenericType<F, R, Args> {
    pub inner: F,
    _args_marker: std::marker::PhantomData<Args>,
    _return_marker: std::marker::PhantomData<R>,
}

/// Turn [`self`] into [`FunctionGenericType`].
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

        impl<F, R0, $($T),*> FromGenericCallable<F, R0, ($($T,)*)> for RustFunction
        where
            R0: crate::object::IntoNyaObject + 'static,
            $($T: $crate::object::FromNyaObject + 'static),*,
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
            R0: $crate::object::IntoNyaObject,
            $($T: $crate::object::FromNyaObject),*
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
