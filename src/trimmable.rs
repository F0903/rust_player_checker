pub trait Trimmable {
	fn trim_newline(&mut self);
}

impl Trimmable for String {
	fn trim_newline(&mut self) {
		if self.ends_with('\n') {
			self.pop();
			if self.ends_with('\r') {
				self.pop();
			}
		}
	}
}
