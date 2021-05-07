use std::io::{ErrorKind, Result};
use winreg::{enums, HKEY};

const BASE_PATH: HKEY = enums::HKEY_CURRENT_USER;
const KEY_PATH: &str = "SOFTWARE\\dr-downloader";

pub fn get_recent() -> Result<String> {
	let key = winreg::RegKey::predef(BASE_PATH).open_subkey(KEY_PATH)?;
	key.get_value("recent_server")
}

pub fn cache_recent(token: impl AsRef<str>) -> Result<()> {
	let (key, _disp) = winreg::RegKey::predef(BASE_PATH).create_subkey(KEY_PATH)?;
	key.set_value("recent_server", &token.as_ref())
}

pub async fn get_or_cache_recent<
	E: std::error::Error + ?Sized,
	T: std::future::Future<Output = std::result::Result<String, Box<E>>>,
	F: Fn() -> T,
>(
	token_factory: F,
) -> Result<String> {
	if let Ok(token) = get_recent() {
		return Ok(token);
	}
	let token = token_factory().await.map_err(|_e| ErrorKind::Other)?;
	cache_recent(&token)?;
	Ok(token)
}
