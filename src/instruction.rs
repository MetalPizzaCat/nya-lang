use crate::object::NyaPrimativeType;

pub enum Instruction {
    Push(NyaPrimativeType),
    Pop,
    SetGlobal(String, NyaPrimativeType),
    RemoveGlobal(String),
    PushGlobal(String),
    PopGlobal(String),
    Add,
}
