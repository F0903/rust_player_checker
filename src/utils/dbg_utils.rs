use std::io::{Result, Write};

fn consume_result<T, E>(res: std::result::Result<T, E>) {
	drop(res);
}

pub fn dump_to_file(list: &[impl std::fmt::Display]) -> Result<()> {
	use std::fs;
	let mut f = fs::File::create("./dbg_dump.txt")?;
	list.iter()
		.for_each(|x| consume_result(writeln!(f, "{}", x)));
	Ok(())
}
