pub trait StringUtils {
	fn trim_newline(&mut self);
}

impl StringUtils for String {
	fn trim_newline(&mut self) {
		if self.ends_with('\n') {
			self.pop();
			if self.ends_with('\r') {
				self.pop();
			}
		}
	}
}
