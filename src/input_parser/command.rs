use super::CallbackFunc;
use std::any::Any;

pub struct Command<'a> {
	args: Vec<&'a str>,
	callback: &'a CallbackFunc,
	passthrough: Option<&'a dyn Any>,
}

impl<'a> Command<'a> {
	pub fn new(callback: &'a CallbackFunc) -> Self {
		Command {
			args: Vec::new(),
			callback,
			passthrough: None,
		}
	}

	pub fn add_arg(mut self, arg: &'a str) -> Self {
		self.args.push(arg);
		self
	}

	pub fn with_passthrough(mut self, passthrough: &'a dyn Any) -> Self {
		self.passthrough = Some(passthrough);
		self
	}

	pub fn get_args(&self) -> &Vec<&'a str> {
		&self.args
	}

	pub fn get_callback(&self) -> &'a CallbackFunc {
		self.callback
	}

	pub fn get_passthrough(&self) -> Option<&'a dyn Any> {
		self.passthrough
	}
}
