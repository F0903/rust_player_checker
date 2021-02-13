use std::fs;
use std::io::{Error, ErrorKind, Result};

const RECENT_PATH: &str = "./recent.txt";

pub fn get_recent() -> Result<String> {
	fs::read_to_string(RECENT_PATH).map_err(|_err| {
		Error::new(
			ErrorKind::NotFound,
			"No recent.txt file was found. Input address manually.",
		)
	})
}

pub fn set_recent(recent: &str) -> Result<()> {
	fs::write(RECENT_PATH, recent)
}
