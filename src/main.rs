use crate::{instruction::Instruction, state::NyaState};

pub mod instruction;
pub mod object;
pub mod state;

fn main() {
    let mut ns = NyaState::new();
    ns.add_constant("variable");
    ns.set_global_direct("variable", "Hello world");
    let program = vec![
        Instruction::GetConst(0),
        Instruction::GetGlobal(0),
        Instruction::Print,
    ];
    ns.run_instructions(&program);
}
