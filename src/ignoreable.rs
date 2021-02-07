pub trait Ignoreable {
	fn ignore(self);
}

impl<T, E> Ignoreable for Result<T, E> {
	fn ignore(self) {}
}
