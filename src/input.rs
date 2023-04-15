use std::f32::consts::TAU;

use bevy::{prelude::*, window::PrimaryWindow};

#[derive(Default, Resource)]
pub struct PlayerInput {
	pub dir: Vec2,
	pub face_turns: f32,
	pub primary: bool,
	pub debug: bool,
	pub full_screen: bool,
}

pub fn sys_player_input(
	mut input: ResMut<PlayerInput>,
	keyboard: Res<Input<KeyCode>>,
	mouse: Res<Input<MouseButton>>,
	q_camera: Query<(&Camera, &GlobalTransform), With<Camera>>,
	q_window: Query<&Window, With<PrimaryWindow>>,
) {
	// direction

	let mut dir = Vec2::ZERO;
	if keyboard.pressed(KeyCode::W) {
		dir.y += 1.0;
	}
	if keyboard.pressed(KeyCode::S) {
		dir.y -= 1.0;
	}
	if keyboard.pressed(KeyCode::A) {
		dir.x -= 1.0;
	}
	if keyboard.pressed(KeyCode::D) {
		dir.x += 1.0;
	}
	input.dir = dir.normalize_or_zero();

	// face

	let (cam, cam_t) = q_camera.single();
	let win = q_window.single();
	let turns = win.cursor_position()
		.and_then(|win_pos| cam.viewport_to_world_2d(cam_t, win_pos))
		.map(|world_pos| world_pos - cam_t.translation().truncate())
		.map(|delta| Vec2::X.angle_between(delta) / TAU);

	if let Some(t) = turns {
		input.face_turns = t;
	}

	// misc

	input.primary = mouse.pressed(MouseButton::Left);

	if keyboard.just_released(KeyCode::F11) {
		input.full_screen = !input.full_screen;
	}

	if keyboard.just_released(KeyCode::F12) {
		input.debug = !input.debug;
	}
}

pub fn sys_gamepad() {
}
