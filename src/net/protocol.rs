use super::msg::Auth;
use naia_shared::{
	derive_channels, Protocolize,
};

#[derive_channels]
pub enum Channels {
	// PlayerCommand,
	// EntityAssignment,
	DeleteMe,
}

#[derive(Protocolize)]
pub enum Protocol {
	Auth(Auth),
}
