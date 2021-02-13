mod ignoreable;
mod input_parser;
mod queryer;
mod recent_provider;
mod string_utils;

use ignoreable::Ignoreable;
use input_parser::{parse_input_args, OptArgs, Replace, ReqArgs};
use queryer::Queryer;
use recent_provider::{get_recent, set_recent};
use std::io::{stdin, stdout, Result, Write};
use std::net::ToSocketAddrs;

#[cfg(windows)]
use winapi::um::playsoundapi;

#[cfg(all(windows, not(debug_assertions)))]
use winapi::um::{consoleapi, errhandlingapi, processenv, winbase, wincon};

//for reference (rustification 2x duo): 51.195.130.177:28235
fn main() -> Result<()> {
	#[cfg(all(windows, not(debug_assertions)))]
	set_win32_color();
	print_start_text();

	loop {
		let cmd_result = parse_input_args(
			stdin().lock(),
			Queryer::new("192.168.1.2:0")?,
			&[
				(
					ReqArgs(&["--listen", "-u"]),
					OptArgs(None),
					Replace(Some(("*recent", || get_recent()))),
					|query, req_args, _opt_args| {
						let server_str = req_args[0];
						let server = server_str
							.to_socket_addrs()?
							.next()
							.expect("Ip could not be parsed.");
						set_recent(server_str)?;
						let name = req_args[1];
						println!("Listening for \"{}\"...", name);
						//TODO: Make it possible to stop listening for a player by pressing a key or typing something.
						loop {
							let players = query.get_players(&server)?;
							if players.iter().any(|x| x.get_name().unwrap() == name) {
								println!("{} IS IN SERVER", name);
								#[cfg(windows)]
								play_alert();
							}

							std::thread::sleep(std::time::Duration::from_millis(60000));
						}
					},
				),
				(
					ReqArgs(&["--dump"]),
					OptArgs(None),
					Replace(Some(("*recent", || get_recent()))),
					|query, req_args, _opt_args| {
						let server_str = req_args[0];
						let server = server_str
							.to_socket_addrs()?
							.next()
							.expect("Ip could not be parsed.");
						let players = query.get_players(&server)?;
						dump_to_file(&players)?;
						set_recent(server_str)
					},
				),
				(
					ReqArgs(&["--print"]),
					OptArgs(None),
					Replace(Some(("*recent", || get_recent()))),
					|query, req_args, _opt_args| {
						let server_str = req_args[0];
						let server = server_str
							.to_socket_addrs()?
							.next()
							.expect("Ip could not be parsed.");
						let players = query.get_players(&server)?;
						players.iter().for_each(|p| println!("{}", p));
						set_recent(server_str)
					},
				),
			],
		);
		if let Err(err) = cmd_result {
			let clear_timer = std::time::Duration::from_secs(5);
			println!("{}\nClearing in {:?}...", err, clear_timer);
			std::thread::sleep(clear_timer);
			clear_terminal();
			print_start_text();
		}
	}
}

fn clear_terminal() {
	print!("\x1B[2J\x1B[1;1H"); //Clear terminal and set cursor to start.
	stdout().flush().unwrap();
}

fn print_start_text() {
	println!(
		"\x1B[1m\x1B[31mRust Player Checker v{}\x1B[0m",
		env!("CARGO_PKG_VERSION")
	);
}

fn dump_to_file(list: &[impl std::fmt::Display]) -> Result<()> {
	use std::fs;
	let mut f = fs::File::create("./dbg_dump.txt")?;
	list.iter().for_each(|x| writeln!(f, "{}", x).ignore());
	Ok(())
}

#[cfg(windows)]
fn play_alert() {
	let bytes = include_bytes!("../media/alert.wav");
	unsafe {
		playsoundapi::PlaySoundA(
			bytes.as_ptr() as *const i8,
			std::ptr::null_mut(),
			playsoundapi::SND_MEMORY,
		);
	}
}

#[cfg(all(windows, not(debug_assertions)))]
fn set_win32_color() {
	// Set virtual console mode to use colors. (for win32)
	unsafe {
		let out = processenv::GetStdHandle(winbase::STD_OUTPUT_HANDLE);
		let mut out_mode: u32 = 0;
		if consoleapi::GetConsoleMode(out, &mut out_mode as *mut u32) == 0 {
			let err = errhandlingapi::GetLastError();
			println!("GetConsoleMode failed with err code {}", err);
		}
		out_mode |= wincon::ENABLE_VIRTUAL_TERMINAL_PROCESSING;
		if consoleapi::SetConsoleMode(out, out_mode) == 0 {
			let err = errhandlingapi::GetLastError();
			println!("SetConsoleMode failed with err code {}", err);
		}

		let inp = processenv::GetStdHandle(winbase::STD_INPUT_HANDLE);
		let mut inp_mode: u32 = 0;
		if consoleapi::GetConsoleMode(inp, &mut inp_mode as *mut u32) == 0 {
			let err = errhandlingapi::GetLastError();
			println!("GetConsoleMode failed with err code {}", err);
		}
		inp_mode |= wincon::ENABLE_VIRTUAL_TERMINAL_INPUT;
		if consoleapi::SetConsoleMode(inp, inp_mode) == 0 {
			let err = errhandlingapi::GetLastError();
			println!("SetConsoleMode failed with err code {}", err);
		}
	}
}
