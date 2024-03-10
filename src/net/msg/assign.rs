use naia_bevy_shared::Message;

#[derive(Debug, Message)]
pub struct Assign {
	pub client_id: u32,
}
