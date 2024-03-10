use naia_bevy_shared::Message;

#[derive(Debug, Message)]
pub struct Input {
	pub velocity_x: f32,
	pub velocity_y: f32,
	pub cursor_dx: f32,
	pub cursor_dy: f32,
	pub primary: bool,
}

#[derive(Debug, Message)]
pub struct InputRepl {
	pub client_id: u32,
	pub velocity_x: f32,
	pub velocity_y: f32,
	pub cursor_dx: f32,
	pub cursor_dy: f32,
	pub primary: bool,
}

impl InputRepl {
	pub fn new(client_id: u32, input: &Input) -> Self {
		Self {
			client_id,
			cursor_dx: input.cursor_dx,
			cursor_dy: input.cursor_dy,
			velocity_x: input.velocity_x,
			velocity_y: input.velocity_y,
			primary: input.primary,
		}
	}
}
