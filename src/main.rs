use crate::state::NyaState;

pub mod instruction;
pub mod object;
pub mod state;

fn main() {
    let mut ns = NyaState::new();
    ns.push_value(0.5);
    ns.push_value("test");
    ns.push_value([0, 1, 4, 5]);
    let n = ns.get_number(-3);

    ns.get_index(-1, 1);
    let i = ns.get_int(-1);

    let s = ns.get_string(-2);
    println!("{s:?}");
    println!("{n:?}");
    println!("{i:?}");
    ns.pop_stack(4);
    ns.set_global("test", [0.5]);
    ns.garbage_collect();
}
