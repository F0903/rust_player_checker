mod ignoreable;
mod input_parser;
mod queryer;
mod string_utils;

use ignoreable::Ignoreable;
use input_parser::parse_input_args;
use queryer::Queryer;
use std::io::{stdin, stdout, Result, Write};
use std::net::ToSocketAddrs;

#[cfg(windows)]
use winapi::um::playsoundapi;

#[cfg(all(windows, not(debug_assertions)))]
use winapi::um::{consoleapi, errhandlingapi, processenv, winbase, wincon};

const START_MSG: &str = "\x1B[1m\x1B[31mRust Player Checker v0.4";

//for reference (rustification 2x duo): 51.195.130.177:28235
fn main() -> Result<()> {
	#[cfg(all(windows, not(debug_assertions)))]
	set_win32_color();
	println!("{}", START_MSG);

	loop {
		let cmd_result = parse_input_args(
			stdin().lock(),
			Queryer::new("192.168.1.2:0")?,
			&[
				(&["--dump"], |query, x| {
					let server = x[0]
						.to_socket_addrs()?
						.next()
						.expect("Ip could not be parsed.");
					let players = query.get_players(&server)?;
					dump_to_file(&players)
				}),
				(&["--print"], |query, x| {
					let server = x[0]
						.to_socket_addrs()?
						.next()
						.expect("Ip could not be parsed.");
					let players = query.get_players(&server)?;
					players.iter().for_each(|p| println!("{}", p));
					Ok(())
				}),
				(&["--listen", "-u"], |query, x| {
					let server = x[0]
						.to_socket_addrs()?
						.next()
						.expect("Ip could not be parsed.");
					let name = x[1];
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
				}),
			],
		);
		if let Err(err) = cmd_result {
			let clear_timer = std::time::Duration::from_secs(5);
			println!("{}\nClearing in {:?}...", err, clear_timer);
			std::thread::sleep(clear_timer);
			print!("\x1B[2J\x1B[1;1H"); //Clear terminal and set cursor to start.
			stdout().flush().unwrap();
			println!("{}", START_MSG);
		}
	}
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
