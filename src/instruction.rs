use crate::object::NyaPrimitiveType;

pub enum Instruction {
    Push(NyaPrimitiveType),
    Pop,
    SetGlobal(String),
    GetGlobal(String),
    Add,
    Halt,
}
