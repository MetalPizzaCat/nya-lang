pub mod functions;
pub mod garbage_collect;
pub mod instruction;
pub mod object;
pub mod state;

#[cfg(test)]
mod tests {
    use crate::{
        functions::{FromGenericCallable, RustClosure},
        garbage_collect::{GcInnerGuard, GcInnerGuardMut},
        state::NyaState,
    };

    #[test]
    fn test_mutable_borrow() {
        let mut ns = NyaState::new();
        ns.create_stack_block();

        ns.push_value("test string");

        {
            let mut s = ns.get_string_mut(-1).unwrap();
            s.push_str(". Whooo!! :3");
        }

        let s2 = ns.get_string(-1).unwrap();

        assert_eq!(*s2, "test string. Whooo!! :3");

        ns.pop_stack(1);
        ns.garbage_collect();
    }

    #[test]
    fn test_functions() {
        let mut ns = NyaState::new();
        ns.create_stack_block();
        ns.push_value("whoo");

        let function = RustClosure::from_callable(
            |mut str: GcInnerGuardMut<String>| -> GcInnerGuardMut<String> {
                str.push_str(" :3");
                str
            },
        );

        function.call(&mut ns);

        let s = ns.get_string(-1).unwrap();
        assert_eq!(*s, "whoo :3");
    }
}
