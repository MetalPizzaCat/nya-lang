use crate::object::NyaPrimitiveType;

pub enum Instruction {
    Push(NyaPrimitiveType),
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
