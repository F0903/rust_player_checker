use super::ignoreable::Ignoreable;
use std::io::{Result, Write};

pub fn dump_to_file(list: &[impl std::fmt::Display]) -> Result<()> {
	use std::fs;
	let mut f = fs::File::create("./dbg_dump.txt")?;
	list.iter().for_each(|x| writeln!(f, "{}", x).ignore());
	Ok(())
}
