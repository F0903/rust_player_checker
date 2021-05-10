mod cacher;
mod input_parser;
mod queryer;
mod utils;

use cacher::{cache_recent, get_recent};
use input_parser::{Command, InputParser, ReplaceFunc};
use queryer::Queryer;
use std::any::*;
use std::io::{stdin, stdout, Error, ErrorKind, Result, Write};
use std::net::ToSocketAddrs;
use utils::{dbg_utils::dump_to_file, trimmable::StringTrimmable};

#[cfg(windows)]
use utils::win_utils::play_alert;
#[cfg(all(windows, not(debug_assertions)))]
use utils::win_utils::set_color_mode;

fn clear_terminal() {
	print!("\x1B[2J\x1B[1;1H"); //Clear terminal and set cursor to start.
	stdout().flush().unwrap();
}

fn print_start_text() {
	fprintln!(
		"\x1B[1m\x1B[31mRust Player Checker v{}\x1B[0m",
		env!("CARGO_PKG_VERSION")
	);
}

fn listen(arg_vals: &[&str], passthrough: Option<&dyn Any>) -> Result<()> {
	let server_str = arg_vals[0];
	let server = server_str
		.to_socket_addrs()?
		.next()
		.expect("Ip could not be parsed.");
	cache_recent(server_str)?;
	let name = arg_vals[1];
	fprintln!(
		"Listening for \"{}\"...\nType '--stop' to stop listening.",
		name
	);

	let val = passthrough.unwrap();
	let query = val.downcast_ref::<Queryer>().ok_or_else(|| {
		Error::new(
			ErrorKind::Other,
			"Could not convert passthrough to Queryer.",
		)
	})?;

	use std::sync::atomic::{AtomicBool, Ordering};
	use std::time::{Duration, Instant};

	let should_stop = AtomicBool::new(false);
	crossbeam_utils::thread::scope(|s| -> Result<()> {
		s.spawn(|_| loop {
			let mut strbuf = String::new();
			stdin().read_line(&mut strbuf).ok();
			let stop = strbuf.trim_newline() == "--stop";
			should_stop.store(stop, Ordering::Relaxed);
			if stop {
				break;
			}
		});

		const DELAY: Duration = Duration::from_secs(30);
		let mut next_scan = Instant::now();
		loop {
			if should_stop.load(Ordering::Relaxed) {
				fprintln!("Stopped listening...");
				break Ok(());
			}

			if Instant::now() < next_scan {
				std::thread::sleep(std::time::Duration::from_millis(1000));
				continue;
			}

			let players = query.get_players(&server)?;
			if players.iter().any(|x| x.get_name().unwrap() == name) {
				fprintln!("{} IS IN SERVER", name);
				for _i in 0..5 {
					#[cfg(windows)]
					play_alert(include_bytes!("../media/alert.wav"));
				}
			}
			next_scan += DELAY;
		}
	})
	.unwrap()
}

fn dump(arg_vals: &[&str], passthrough: Option<&dyn Any>) -> Result<()> {
	let server_str = arg_vals[0];
	let server = server_str
		.to_socket_addrs()?
		.next()
		.expect("Ip could not be parsed.");
	let val = passthrough.unwrap();
	let query = val.downcast_ref::<Queryer>().ok_or_else(|| {
		Error::new(
			ErrorKind::Other,
			"Could not convert passthrough to Queryer.",
		)
	})?;
	let players = query.get_players(&server)?;
	dump_to_file(&players)?;
	fprintln!("Dumped playerlist to dbg_dump.txt");
	cache_recent(server_str)
}

fn print(arg_vals: &[&str], passthrough: Option<&dyn Any>) -> Result<()> {
	let server_str = arg_vals[0];
	let server = server_str
		.to_socket_addrs()?
		.next()
		.expect("Ip could not be parsed.");

	let val = passthrough.unwrap();
	let query = val.downcast_ref::<Queryer>().ok_or_else(|| {
		Error::new(
			ErrorKind::Other,
			"Could not convert passthrough to Queryer.",
		)
	})?;
	let players = query.get_players(&server)?;
	players.iter().for_each(|p| println!("{}", p));

	cache_recent(server_str)
}

fn main() -> Result<()> {
	#[cfg(all(windows, not(debug_assertions)))]
	set_color_mode();
	print_start_text();

	let query = Queryer::new("192.168.1.2:0")?;

	let listen_command = Command::new(listen)
		.add_arg("--listen")
		.add_arg("-u")
		.with_passthrough(&query);
	let print_command = Command::new(print)
		.add_arg("--print")
		.with_passthrough(&query);
	let dump_command = Command::new(dump)
		.add_arg("--dump")
		.with_passthrough(&query);
	let parser = InputParser::new()
		.add_command(listen_command)
		.add_command(print_command)
		.add_command(dump_command)
		.add_replacement_var("*recent", &(get_recent as ReplaceFunc));

	// If program is called from commandline with args.
	let args = std::env::args();
	let arg_len = args.len();
	let has_args = arg_len > 1;
	if has_args {
		//TODO: Check if the first arg is ever the exe path.
		let input = args.fold(String::with_capacity(arg_len * 5), |s, arg| {
			s + &(arg + " ")
		});
		parser.parse_from_string(input)?;
		return Ok(());
	}

	loop {
		fprint!("-> ");
		let cmd_result = parser.parse_from_stdin();
		if let Err(err) = cmd_result {
			let clear_timer = std::time::Duration::from_secs(5);
			fprintln!("{}\nClearing in {:?}...", err, clear_timer);
			std::thread::sleep(clear_timer);
			clear_terminal();
		}
		fprintln!();
	}
}
