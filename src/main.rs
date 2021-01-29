use a2s::A2SClient;
use std::io::{stdin, stdout, BufRead, Write};
use winapi::um::playsoundapi;

//TODO: make own impl instead of lib?
//for reference (rustification 2x duo): 51.195.130.177:28235
fn main() {
	let mut out = stdout();
	out.lock()
		.by_ref()
		.write_all(b"Please enter the username to check for: ")
		.expect("Could not write to stdout.");
	out.flush().expect("Could not flush.");

	let inp = stdin();
	let mut name_to_check = String::new();
	let read = inp
		.lock()
		.read_line(&mut name_to_check)
		.expect("Could not get input.");
	name_to_check = name_to_check[..read].to_owned();

	let client = A2SClient::new().expect("Could not start client.");
	loop {
		let players = client
			.players("51.195.130.177:28235")
			.expect("Could not get players.");

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
