pub mod functions;
pub mod garbage_collector;
pub mod instruction;
pub mod object;
pub mod state;

pub mod codegen;
#[cfg(test)]
mod tests {
    use crate::{
        functions::{FromGenericCallable, RustFunction},
        garbage_collector::{GcInnerGuard, GcInnerGuardMut},
        instruction::Instruction,
        object::{FromNyaObject, IntoNyaObject, NyaPrimitiveObject},
        state::NyaState,
    };

    #[test]
    fn test_mutable_borrow() {
        let mut ns = NyaState::new();
        ns.create_stack_frame();

        ns.push_value("test string");

        {
            let mut s: GcInnerGuardMut<String> = ns.get_stack(-1).unwrap();
            s.push_str(". Whooo!! :3");
        }

        let s2: GcInnerGuard<String> = ns.get_stack(-1).unwrap();

        assert_eq!(*s2, "test string. Whooo!! :3");

        ns.pop_stack(1);
        ns.garbage_collect();
    }

    #[test]
    fn test_functions() {
        let mut ns = NyaState::new();
        ns.create_stack_frame();
        ns.push_value("whoo");

        let function = RustFunction::from_callable(
            |mut str: GcInnerGuardMut<String>| -> GcInnerGuardMut<String> {
                str.push_str(" :3");
                str
            },
        );

        function.call(&mut ns);

        let s: GcInnerGuard<String> = ns.get_stack(-1).unwrap();
        assert_eq!(*s, "whoo :3");
    }

    #[test]
    fn test_instruction_set() {
        let mut ns = NyaState::new();
        let str = "Hello world".into_nya_object(&mut ns);
        let str_name = "global1".into_nya_object(&mut ns);
        ns.add_constant(str_name);
        ns.run_instructions(
            Vec::new(),
            &vec![Instruction::Push(str), Instruction::SetGlobal(0)],
        );
        let v = ns.get_global_direct("global1");
        assert!(v.is_some());

        let str_val = GcInnerGuard::from_nya_object(v.unwrap());
        assert!(str_val.is_some());
        assert_eq!(*str_val.unwrap(), "Hello world".to_owned());
    }
}
