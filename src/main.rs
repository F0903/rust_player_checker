mod cacher;
mod input_parser;
mod queryer;
mod utils;

use cacher::{cache_recent, get_recent};
use input_parser::{CallbackFunc, Command, InputParser, ReplaceFunc};
use queryer::Queryer;
use std::any::*;
use std::io::{stdin, stdout, Error, ErrorKind, Result, Write};
use std::net::ToSocketAddrs;
use utils::dbg_utils::dump_to_file;

#[cfg(windows)]
use utils::win_utils::play_alert;
#[cfg(all(windows, not(debug_assertions)))]
use utils::win_utils::set_color_mode;

fn clear_terminal() {
	print!("\x1B[2J\x1B[1;1H"); //Clear terminal and set cursor to start.
	stdout().flush().unwrap();
}

fn print_start_text() {
	println!(
		"\x1B[1m\x1B[31mRust Player Checker v{}\x1B[0m",
		env!("CARGO_PKG_VERSION")
	);
	stdout().flush().unwrap();
}

fn listen(arg_vals: &[&str], passthrough: Option<&dyn Any>) -> Result<()> {
	let server_str = arg_vals[0];
	let server = server_str
		.to_socket_addrs()?
		.next()
		.expect("Ip could not be parsed.");
	cache_recent(server_str)?;
	let name = arg_vals[1];
	println!("Listening for \"{}\"...", name);

	let query = passthrough
		.unwrap()
		.downcast_ref::<Queryer>()
		.ok_or_else(|| {
			Error::new(
				ErrorKind::Other,
				"Could not convert passthrough to Queryer.",
			)
		})?;

	//TODO: Make it possible to stop listening for a player by pressing a key or typing something.
	loop {
		let players = query.get_players(&server)?;
		if players.iter().any(|x| x.get_name().unwrap() == name) {
			println!("{} IS IN SERVER", name);
			#[cfg(windows)]
			play_alert(include_bytes!("../media/alert.wav"));
		}

		std::thread::sleep(std::time::Duration::from_millis(60000));
	}
}

fn dump(arg_vals: &[&str], passthrough: Option<&dyn Any>) -> Result<()> {
	let server_str = arg_vals[0];
	let server = server_str
		.to_socket_addrs()?
		.next()
		.expect("Ip could not be parsed.");
	let query = passthrough
		.unwrap()
		.downcast_ref::<Queryer>()
		.ok_or_else(|| {
			Error::new(
				ErrorKind::Other,
				"Could not convert passthrough to Queryer.",
			)
		})?;
	let players = query.get_players(&server)?;
	dump_to_file(&players)?;
	cache_recent(server_str)
}

fn print(arg_vals: &[&str], passthrough: Option<&dyn Any>) -> Result<()> {
	let server_str = arg_vals[0];
	let server = server_str
		.to_socket_addrs()?
		.next()
		.expect("Ip could not be parsed.");

	let query = passthrough
		.unwrap()
		.downcast_ref::<Queryer>()
		.ok_or_else(|| {
			Error::new(
				ErrorKind::Other,
				"Could not convert passthrough to Queryer.",
			)
		})?;
	let players = query.get_players(&server)?;
	players.iter().for_each(|p| println!("{}", p));

	cache_recent(server_str)
}

//for reference (rustification 2x duo): 51.195.130.177:28235
fn main() -> Result<()> {
	#[cfg(all(windows, not(debug_assertions)))]
	set_color_mode();
	print_start_text();

	let query = Queryer::new("192.168.1.2:0")?;
	let listen_command = Command::new(listen as CallbackFunc)
		.add_arg("--listen")
		.add_arg("-u")
		.with_passthrough(&query);
	let print_command = Command::new(print as CallbackFunc)
		.add_arg("--print")
		.with_passthrough(&query);
	let dump_command = Command::new(dump as CallbackFunc)
		.add_arg("--dump")
		.with_passthrough(&query);
	let parser = InputParser::new()
		.add_command(listen_command)
		.add_command(print_command)
		.add_command(dump_command)
		.add_replacement_var("*recent", &((|| get_recent()) as ReplaceFunc));

	loop {
		let cmd_result = parser.parse(stdin().lock());
		if let Err(err) = cmd_result {
			let clear_timer = std::time::Duration::from_secs(5);
			println!("{}\nClearing in {:?}...", err, clear_timer);
			std::thread::sleep(clear_timer);
			clear_terminal();
			print_start_text();
		}
	}
}
