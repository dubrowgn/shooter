use naia_bevy_shared::Message;

#[derive(Message)]
pub struct Auth {
	pub token: String,
}

impl Auth {
	pub fn new(token: &str) -> Self {
		Auth{ token: token.to_string() }
	}
}
