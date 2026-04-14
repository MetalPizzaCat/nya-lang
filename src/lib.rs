pub mod garbage_collect;
pub mod instruction;
pub mod object;
pub mod state;

#[cfg(test)]
mod tests {
    use crate::state::NyaState;

    #[test]
    fn test_mutable_borrow() {
        let mut ns = NyaState::new();
        ns.create_stack_block();

        ns.push_value("test string");

        // {
        //     let mut s = ns.get_string_mut(-1).unwrap();
        //     s.push_str(". Whooo!! :3");
        // }
        //
        // // let s2 = ns.get_string(-1).unwrap();
        // // println!("{}", *s2);
        // ns.pop_stack(1);
        ns.garbage_collect();
    }
}
