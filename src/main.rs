mod ignoreable;
mod queryer;
mod trimmable;

use ignoreable::Ignoreable;
use queryer::Queryer;
use std::io::{stdin, stdout, BufRead, Read, Result, Write};
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use trimmable::Trimmable;
use winapi::um::playsoundapi;

//TODO: Perhaps parse a config file, instead of asking for the ip and username evertime? Or instead use cmd-line args?

//for reference (rustification 2x duo): 51.195.130.177:28235
fn main() -> Result<()> {
	println!("Rust Player Checker v0.3");

	let out = stdout();
	let inp = stdin();

	let queryer = Queryer::new("192.168.1.2:0")?;

	parse_input_args(
		(inp.lock().by_ref(), out.lock().by_ref()),
		queryer,
		&[
			//(&["--dump"], |io, query, x| {}),
			(&["", "-s", "-u"], |_io, query, x| {
				let server = str_to_socket_addr(x[0]);
				let name = x[1];
				println!("Starting search for \"{}\"...", name);
				loop {
					let players = query.get_players(&server).unwrap();
					dump_to_file(&players).unwrap();
					if players.iter().any(|x| x.get_name().unwrap() == name) {
						println!("{} IS IN SERVER", name);
						play_alert();
					}

					std::thread::sleep(std::time::Duration::from_millis(60000));
				}
			}),
		],
	)?;
	Ok(())
}

fn parse_input_args<I: BufRead, O: Write>(
	mut io: (&mut I, &mut O),
	queryer: Queryer,
	funcs: &[(&[&str], fn(&mut (&mut I, &mut O), &Queryer, &[&str]))],
) -> Result<()> {
	let mut strbuf = String::with_capacity(10);
	let read = io.0.read_line(&mut strbuf)?;
	let input = &strbuf[..read];
	for (args, func) in funcs {
		if !input.contains(args[0]) {
			continue;
		}

		let mut vals = Vec::new();
		for arg in *args {
			if *arg == "" {
				continue;
			}
			let mut splits = input.split(arg);
			let val = splits
				.nth(1)
				.expect("Could not get value from split.")
				.trim_start();

			let mut start = 0;
			let mut end = val
				.find(&[' ', '\r', '\n', '\0'][..])
				.unwrap_or_else(|| val.len());

			let matches = val.matches('"').count();
			if matches == 2 {
				let first = val.find('"').unwrap() + 1;
				let last = val.rfind('"').unwrap();
				if first < end {
					start = first;
					end = last;
				}
			} else if matches == 1 || matches > 2 {
				panic!("An argument was found specified with only one or more than 2 \" when 2 was needed.");
			}

			vals.push(&val[start..end]);
		}

		func(&mut io, &queryer, &vals);
	}
	Ok(())
}

fn dump_to_file(list: &[impl std::fmt::Display]) -> Result<()> {
	use std::fs;
	let mut f = fs::File::create("./dbg_dump.txt")?;
	list.iter().for_each(|x| writeln!(f, "{}", x).ignore());
	Ok(())
}

fn str_to_socket_addr(inp: &str) -> SocketAddr {
	let mut split = inp.split(':');
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
