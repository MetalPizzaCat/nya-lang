use crate::object::NyaPrimitiveObject;

pub enum Instruction {
    Push(NyaPrimitiveObject),
    Pop,
    SetGlobal(usize),
    GetGlobal(usize),
    GetConst(usize),
    GetLocal(usize),
    SetLocal(usize),
    Add,
    Print,
    Halt,
}
