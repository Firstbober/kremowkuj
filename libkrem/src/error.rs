#[derive(Clone, Debug)]
pub struct Position {
	pub line: i32,
	pub column: i32
}

#[derive(Clone, Debug)]
pub struct Error<T> {
	pub position: Position,
	pub kind: T,
}

pub trait Info {
	fn get_message(&self) -> &'static str;
	fn get_suggestion(&self) -> &'static str;
}

impl<T> Info for Error<T> {
	default fn get_message(&self) -> &'static str {
		unimplemented!();
	}

	default fn get_suggestion(&self) -> &'static str {
		unimplemented!();
	}
}