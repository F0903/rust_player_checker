use std::io::Result;
use winreg::{enums, HKEY};

const BASE_PATH: HKEY = enums::HKEY_CURRENT_USER;
const KEY_PATH: &str = "SOFTWARE\\rust_player_checker";

pub fn get_recent() -> Result<String> {
	let key = winreg::RegKey::predef(BASE_PATH).open_subkey(KEY_PATH)?;
	key.get_value("recent_server")
}

pub fn cache_recent(token: impl AsRef<str>) -> Result<()> {
	let (key, _disp) = winreg::RegKey::predef(BASE_PATH).create_subkey(KEY_PATH)?;
	key.set_value("recent_server", &token.as_ref())
}
