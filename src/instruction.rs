use crate::object::NyaPrimativeType;

pub enum Instruction {
    Push(NyaPrimativeType),
    Pop,
    SetGlobal(String),
    GetGlobal(String),
    Add,
    Halt,
}
