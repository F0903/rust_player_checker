use std::any::Any;
use std::io::Result;

type ArgValues<'a> = &'a [&'a str];
pub type ReplaceFunc = fn() -> Result<String>;
pub type CallbackFunc = fn(ArgValues, Option<&dyn Any>) -> Result<()>;

pub mod command;
pub use command::Command;

pub mod parser;
pub use parser::InputParser;
