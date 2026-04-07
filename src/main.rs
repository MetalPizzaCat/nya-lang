use crate::state::NyaState;

pub mod object;
pub mod state;

fn main() {
    let mut ns = NyaState::new();
    ns.push_value(0.5);
    ns.push_value("test");
    ns.push_value([0, 1, 4, 5]);
    let n = ns.get_number(-2);
    let s = ns.get_string(-1);
    println!("{s:?}");
    println!("{n:?}");
    ns.pop_stack(2);
    ns.garbage_collect();
}
