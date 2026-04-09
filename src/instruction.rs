use crate::object::NyaPrimitiveType;

pub enum Instruction {
    Push(NyaPrimitiveType),
    Pop,
    SetGlobal(usize),
    GetGlobal(usize),
    GetConst(usize),
    GetLocal,
    SetLocal,
    Add,
    Print,
    Halt,
}
