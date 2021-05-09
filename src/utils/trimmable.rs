pub trait StringTrimmable {
	fn trim_newline(&mut self) -> &mut Self;
}

impl StringTrimmable for String {
	fn trim_newline(&mut self) -> &mut Self {
		if self.ends_with('\n') {
			self.pop();
			if self.ends_with('\r') {
				self.pop();
			}
		}
		self
	}
}

pub trait StrTrimmable {
	fn trim_newline(&self) -> &str;
}

impl StrTrimmable for &str {
	fn trim_newline(&self) -> &str {
		let mut end = self.len();
		self.chars().rev().for_each(|ch| {
			if ch == '\n' || ch == '\r' {
				end -= 1;
			}
		});
		&self[..end]
	}
}
