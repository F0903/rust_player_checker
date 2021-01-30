mod strutil;

use a2s::A2SClient;
use std::io::{stdin, stdout, BufRead, Read, Write};
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use strutil::Trimmable;
use winapi::um::playsoundapi;

//TODO: Perhaps make own impl of Source Query, looks to be simple for this usecase.
//TODO: Perhaps parse a config file, instead of asking for the ip and username evertime? Or instead use cmd-line args?

//for reference (rustification 2x duo): 51.195.130.177:28235
fn main() {
	let out = stdout();
	let inp = stdin();

	let server = get_server(inp.lock().by_ref(), out.lock().by_ref());
	let mut name_to_check = get_user(inp.lock().by_ref(), out.lock().by_ref());
	name_to_check.trim_newline();

	let client = A2SClient::new().expect("Could not start client.");
	loop {
		let players = client.players(server).expect("Could not get players.");

		let player_list = players.players;

		if player_list.iter().any(|x| x.name == name_to_check) {
			println!("{} IS IN SERVER", name_to_check);
			play_sound_cue();
		} else {
			println!("{} is not currently in server", name_to_check);
		}
		std::thread::sleep(std::time::Duration::from_millis(30000));
	}
}

fn get_server(inp: &mut impl BufRead, out: &mut impl Write) -> SocketAddr {
	out.write_all(b"Please enter the server ip and port: ")
		.expect("Could not write to stdout.");
	out.flush().expect("Could not flush.");
	let mut strbuf = String::new();
	inp.read_line(&mut strbuf).expect("Could not read input.");

	let mut split = strbuf.split(':');
	let ip_str = split.next().expect("Could not get ip from input.");
	let mut port_str = split
		.next()
		.expect("Could not get port from input.")
		.to_owned();
	port_str.trim_newline();

	SocketAddr::new(
		IpAddr::from_str(ip_str).expect("Could not parse ip from input."),
		port_str
			.parse::<u16>()
			.expect("Could not parse port number from input."),
	)
}

fn get_user(inp: &mut impl BufRead, out: &mut impl Write) -> String {
	out.write_all(b"Please enter the username to check for: ")
		.expect("Could not write to stdout.");
	out.flush().expect("Could not flush.");
	let mut strbuf = String::new();
	let read = inp.read_line(&mut strbuf).expect("Could not get input.");
	strbuf[..read].to_owned()
}

#[cfg(windows)]
fn play_sound_cue() {
	let bytes = include_bytes!("../media/alert.wav");
	unsafe {
		playsoundapi::PlaySoundA(
			bytes.as_ptr() as *const i8,
			std::ptr::null_mut(),
			playsoundapi::SND_MEMORY,
		);
	}
}
