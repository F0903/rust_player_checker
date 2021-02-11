mod ignoreable;
mod input_parser;
mod queryer;
mod string_utils;

use ignoreable::Ignoreable;
use input_parser::parse_input_args;
use queryer::Queryer;
use std::io::{stdin, stdout, Result, Write};
use std::net::ToSocketAddrs;
use winapi::um::playsoundapi;

const START_MSG: &str = "Rust Player Checker v0.4";

//for reference (rustification 2x duo): 51.195.130.177:28235
fn main() -> Result<()> {
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
