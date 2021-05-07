struct PlayerListener {
	listening: bool,
}

//TODO: Finish refactoring stuff into this.
impl PlayerListener {
	pub fn start() -> PlayerListener {}

	pub fn stop(&mut self) {
		self.listening = false;
	}
}
