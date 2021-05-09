use super::{command::Command, ReplaceFunc};
use std::io::{BufRead, Error, ErrorKind, Result};

pub struct InputParser<'a> {
	commands: Vec<Command<'a>>,
	replacement_vars: Vec<(&'a str, &'a ReplaceFunc)>,
}

impl<'a> InputParser<'a> {
	pub fn new() -> InputParser<'a> {
		InputParser {
			commands: Vec::new(),
			replacement_vars: Vec::new(),
		}
	}

	pub fn add_command(mut self, arg: Command<'a>) -> Self {
		self.commands.push(arg);
		self
	}

	pub fn add_replacement_var(mut self, var: &'a str, replace_callback: &'a ReplaceFunc) -> Self {
		self.replacement_vars.push((var, replace_callback));
		self
	}

	pub fn parse(&self, mut input: impl BufRead) -> Result<()> {
		let mut strbuf = String::with_capacity(10);
		let read = input.read_line(&mut strbuf)?;
		let input = &strbuf[..read];
		self.execute_commands(input)?;
		Ok(())
	}

	fn execute_commands(&self, input: &str) -> Result<()> {
		let cmds = &self.commands;
		for cmd in cmds {
			let mut contained_arg = false;
			let args = &cmd.get_args();
			let mut vals = Vec::<String>::with_capacity(args.len());
			for arg in *args {
				if !input.contains(arg) {
					continue;
				}
				let arg_value = Self::parse_arg_value(input, arg, &self.replacement_vars)?;
				vals.push(arg_value);
				contained_arg = true;
			}
			if !contained_arg {
				continue;
			}
			cmd.get_callback()(
				&vals.iter().map(|x| &x[..]).collect::<Vec<&str>>(),
				cmd.get_passthrough(),
			)?;
		}
		Ok(())
	}

	fn parse_arg_value(
		input: &str,
		arg: &str,
		replacement_vars: &[(&'a str, &'a ReplaceFunc)],
	) -> Result<String> {
		let mut splits = input.split(arg);
		let val = splits
			.nth(1)
			.ok_or_else(|| {
				Error::new(
					ErrorKind::InvalidInput,
					"Arg value could not be parsed from input.",
				)
			})?
			.trim_start();

		let mut start = 0;
		let mut end = val
			.find(&[' ', '\r', '\n', '\0'][..])
			.unwrap_or_else(|| val.len());

		let matches = val.matches('"').count();
		if matches >= 2 {
			let first = val.find('"').unwrap() + 1;
			let last = val[first + 1..].find('"').unwrap() + 2;
			if first < end {
				start = first;
				end = last;
			}
		} else if matches == 1 {
			return Err(Error::new(
				ErrorKind::InvalidInput,
				"An argument was found specified with only one ' \" ' when 2 was needed.",
			));
		}

		let mut val = String::from(&val[start..end]);
		// Parse replacement variables.
		for rep in replacement_vars {
			let rep_res = &rep.1();
			if let Ok(rep_val) = rep_res {
				val = val.replace(rep.0, rep_val);
			}
		}

		Ok(val)
	}
}
