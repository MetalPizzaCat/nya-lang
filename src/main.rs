use crate::{
    functions::{CallableRustClosure, FromGenericCallable, IntoFunctionGenericType, RustClosure},
    instruction::Instruction,
    object::IntoNyaType,
    state::NyaState,
};

pub mod functions;
pub mod instruction;
pub mod object;
pub mod state;

fn main() {
    let mut ns = NyaState::new();
    ns.create_stack_block();

    ns.push_value(105);
    ns.push_value(15);

    let clos = RustClosure::from_callable(|a: i64, b: i64| a + b);

    clos.call(&mut ns).unwrap();
    let result: i64 = ns.get_stack(-1).unwrap();
    println!("{result}");

    return;

    ns.add_constant("variable");
    ns.set_global_direct("variable", "Hello world");
    let program = vec![
        Instruction::GetConst(0),
        Instruction::GetGlobal(0),
        Instruction::Print,
    ];
    ns.run_instructions(Vec::new(), &program);
}
