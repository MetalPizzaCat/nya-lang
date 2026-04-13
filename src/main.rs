use crate::{instruction::Instruction, state::NyaState};

pub mod garbage_collect;
pub mod instruction;
pub mod object;
pub mod state;

fn main() {
    let mut ns = NyaState::new();
    ns.create_stack_block();

    ns.push_value("test string");

    {
        let mut s = ns.get_string_mut(-1).unwrap();
        s.push_str(". Whooo!! :3");
        drop(s);
    }

    let s2 = ns.get_string(-1).unwrap();
    println!("{}", *s2);

    ns.pop_stack(1);
    ns.garbage_collect();

    // ns.add_constant("variable");
    // ns.set_global_direct("variable", "Hello world");
    // let program = vec![
    //     Instruction::GetConst(0),
    //     Instruction::GetGlobal(0),
    //     Instruction::Print,
    // ];
    // ns.run_instructions(Vec::new(), &program);
}
