use crate::queryer::Queryer;
use std::io::{BufRead, Error, ErrorKind, Result};

type ArgList<'a> = &'a [&'a str];
type CallbackFunc = fn(&Queryer, ArgList, Option<ArgList>) -> Result<()>;
type ReplaceFunc = fn() -> Result<String>;

pub struct ReqArgs<'a>(pub ArgList<'a>); // Required args.
pub struct OptArgs<'a>(pub Option<ArgList<'a>>); // Optional args.
pub struct Replace<'a>(pub Option<(&'a str, ReplaceFunc)>); // Replace vars. (currently only supports one var for simplicitys sake, but its easy to add more.)

//TODO: Refactor this method into a struct with optional args.
pub fn parse_input_args(
	mut inp: impl BufRead,
	queryer: Queryer,
	funcs: &[(ReqArgs, OptArgs, Replace, CallbackFunc)],
) -> Result<()> {
	let mut strbuf = String::with_capacity(10);
	let read = inp.read_line(&mut strbuf)?;
	let input = &strbuf[..read];
	for (req_args, opt_args, rep_vars, func) in funcs {
		let req = req_args.0;
		let opt = opt_args.0;

		let first_val = req[0];
		if first_val == "" {
			continue;
		}
		if !input.contains(first_val) {
			continue;
		}

		let req_vals = parse_req_args(&req, rep_vars, input)?;
		let opt_vals = None;
		if let Some(o) = opt {
			parse_opt_args(&o, rep_vars, input)?;
		}

		func(
			&queryer,
			&req_vals.iter().map(|x| &x[..]).collect::<Vec<&str>>(), // This does not look like the most efficient thing in the world
			opt_vals,
		)?;
		return Ok(());
	}
	Err(Error::new(
		ErrorKind::InvalidInput,
		"Command not recognized.",
	))
}

fn parse_arg<'a>(arg: &'a str, rep_vars: &Replace, input: &'a str) -> Result<String> {
	let mut splits = input.split(arg);
	let mut val = splits
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

	val = &val[start..end];
	if let Some(rep) = rep_vars.0 {
		if val.matches(rep.0).count() >= 1 {
			return Ok(val.replace(rep.0, &rep.1()?));
		}
	}
	Ok(val.to_owned())
}

fn parse_req_args<'a>(
	req_args: &'a ArgList,
	rep_vars: &Replace,
	input: &'a str,
) -> Result<Vec<String>> {
	let mut req_vals = Vec::with_capacity(req_args.len());
	for arg in *req_args {
		if *arg == "" {
			continue;
		}
		let val = parse_arg(arg, rep_vars, input)?;
		req_vals.push(val);
	}
	Ok(req_vals)
}

fn parse_opt_args<'a>(
	opt_args: &'a ArgList,
	rep_vars: &Replace,
	input: &'a str,
) -> Result<Vec<String>> {
	let mut opt_vals = Vec::with_capacity(opt_args.len());
	for arg in *opt_args {
		if *arg == "" {
			continue;
		}
		if let Ok(val) = parse_arg(arg, rep_vars, input) {
			opt_vals.push(val);
		}
	}
	Ok(opt_vals)
}
