mod input_parser;
mod queryer;
mod recent_provider;
mod utils;

use utils::dbg_utils::dump_to_file;

use input_parser::{parse_input_args, OptArgs, Replace, ReqArgs};
use queryer::Queryer;
use recent_provider::{get_recent, set_recent};
use std::future::*;
use std::io::{stdin, stdout, Result, Write};
use std::net::ToSocketAddrs;

#[cfg(windows)]
use utils::win_utils::play_alert;
#[cfg(all(windows, not(debug_assertions)))]
use utils::win_utils::set_color_mode;

//for reference (rustification 2x duo): 51.195.130.177:28235
fn main() -> Result<()> {
	#[cfg(all(windows, not(debug_assertions)))]
	set_color_mode();
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
								play_alert(include_bytes!("../media/alert.wav"));
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
