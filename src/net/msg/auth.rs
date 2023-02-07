use bevy::ecs::component::Component;
use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::net::protocol::Protocol"]
pub struct Auth {
	pub token: Property<String>,
}

impl Auth {
	pub fn new(token: &str) -> Self {
		Auth::new_complete(token.to_string())
	}
}
