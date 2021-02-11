use crate::queryer::Queryer;
use std::io::{BufRead, Error, ErrorKind, Result};

type CallbackFunc = fn(&Queryer, &[&str]) -> Result<()>;

pub fn parse_input_args(
	mut inp: impl BufRead,
	queryer: Queryer,
	funcs: &[(&[&str], CallbackFunc)],
) -> Result<()> {
	let mut strbuf = String::with_capacity(10);
	let read = inp.read_line(&mut strbuf)?;
	let input = &strbuf[..read];
	for (args, func) in funcs {
		if args[0] == "" {
			continue;
		}

		if !input.contains(args[0]) {
			continue;
		}

		let mut vals = Vec::with_capacity(args.len());
		for arg in *args {
			if *arg == "" {
				continue;
			}
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
			if matches == 2 {
				let first = val.find('"').unwrap() + 1;
				let last = val.rfind('"').unwrap();
				if first < end {
					start = first;
					end = last;
				}
			} else if matches == 1 || matches > 2 {
				return Err(Error::new(ErrorKind::InvalidInput, "An argument was found specified with only one or more than 2 ' \" ' when 2 was needed."));
			}

			vals.push(&val[start..end]);
		}
		func(&queryer, &vals)?;
		return Ok(());
	}
	Err(Error::new(
		ErrorKind::InvalidInput,
		"Command not recognized.",
	))
}
