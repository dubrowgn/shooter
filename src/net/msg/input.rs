use naia_bevy_shared::Message;

#[derive(Debug, Message)]
pub struct Input {
	pub velocity_x: f32,
	pub velocity_y: f32,
	pub cursor_dx: f32,
	pub cursor_dy: f32,
	pub primary: bool,
}
