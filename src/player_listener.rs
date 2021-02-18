struct PlayerListener {
	listening: bool,
}

impl PlayerListener {
	pub fn start() -> PlayerListener {}

	pub fn stop(&mut self) {
		self.listening = false;
	}
}
