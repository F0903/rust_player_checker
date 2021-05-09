use super::{command::Command, ReplaceFunc};
use std::io::{Error, ErrorKind, Result};

struct ReplacePair<'a> {
	var_name: &'a str,
	replace_callback: &'a ReplaceFunc,
}

pub struct InputParser<'a> {
	commands: Vec<Command<'a>>,
	replacement_vars: Vec<ReplacePair<'a>>,
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

	pub fn add_replacement_var(
		mut self,
		var_name: &'a str,
		replace_callback: &'a ReplaceFunc,
	) -> Self {
		self.replacement_vars.push(ReplacePair {
			var_name,
			replace_callback,
		});
		self
	}

	pub fn parse_from_stdin(&self) -> Result<()> {
		let mut strbuf = String::with_capacity(10);
		let read;
		{
			read = std::io::stdin().read_line(&mut strbuf)?;
		}
		let content = &strbuf[..read];
		self.execute_commands(content)?;
		Ok(())
	}

	pub fn parse_from_string(&self, input: impl AsRef<str>) -> Result<()> {
		self.execute_commands(input.as_ref())
	}

	fn execute_commands(&self, input: &str) -> Result<()> {
		for cmd in &self.commands {
			let mut contained_arg = false;
			let args = &cmd.get_args();
			let mut vals = Vec::<String>::with_capacity(args.len());
			for arg in *args {
				if !input.contains(arg) {
					continue;
				}
				let arg_value = self.parse_arg_value(input, arg)?;
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

	fn replace_with_vars(&self, input: &mut String) {
		for rep in &self.replacement_vars {
			let callback = rep.replace_callback;
			let rep_res = callback();
			if let Ok(rep_val) = rep_res {
				*input = input.replace(rep.var_name, &rep_val);
			}
		}
	}

	fn parse_arg_value(&self, input: &str, arg: &str) -> Result<String> {
		let mut splits = input.split(arg);
		let arg_value = splits
			.nth(1)
			.ok_or_else(|| {
				Error::new(
					ErrorKind::InvalidInput,
					"Arg value could not be parsed from input.",
				)
			})?
			.trim_start();

		let mut start = 0;
		let mut end = arg_value
			.find(&[' ', '\r', '\n', '\0'][..])
			.unwrap_or_else(|| arg_value.len());

		let matches = arg_value.matches('"').count();
		if matches >= 2 {
			let first = arg_value.find('"').unwrap() + 1;
			let last = arg_value[first + 1..].find('"').unwrap() + 2;
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

		let mut final_arg_value = String::from(&arg_value[start..end]);
		self.replace_with_vars(&mut final_arg_value);
		Ok(final_arg_value)
	}
}
