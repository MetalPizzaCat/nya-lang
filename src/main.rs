use crate::state::NyaState;

pub mod instruction;
pub mod object;
pub mod state;

fn main() {
    let mut ns = NyaState::new();
    ns.push_value(0.5);
    ns.push_value("test");
    ns.push_value([0, 1, 4, 5]);
    let n = ns.to_number(-3);

    ns.get_field(-1, 1);
    let i = ns.to_int(-1);

    let idx = ns.add_constant("blah");
    ns.get_constant(idx);

    let s = ns.to_string(-4);
    let s2 = ns.to_string(-1);
    println!("{s2:?}");
    println!("{s:?}");
    println!("{n:?}");
    println!("{i:?}");
    ns.pop_stack(4);
    ns.set_global_direct("test", [0.5]);
    ns.garbage_collect();
}
