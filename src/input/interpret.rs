use bevy::{
	input::{
		gamepad::{
			GamepadButton,
			GamepadEvent,
		},
		keyboard::KeyboardInput,
	},
	prelude::*,
	window::PrimaryWindow,
};
use std::f32::consts::TAU;
use super::tick_input_plugin::{
	Gamepad,
	Keyboard,
	Mouse,
};

#[derive(Default, Reflect)]
pub enum InputType {
	#[default]
	Keyboard,
	Gamepad,
}

#[derive(Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct PlayerInput {
	pub input_type: InputType,
	pub dir: Vec2,
	pub face_turns: f32,
	pub primary: bool,
	pub debug: bool,
	pub full_screen: bool,
}

pub fn sys_input_type	(
	mut input: ResMut<PlayerInput>,
	mut keyboard_input_events: EventReader<KeyboardInput>,
	mut gamepad_events: EventReader<GamepadEvent>,
	gamepads: Res<Gamepads>,
) {
	for gamepad_event in gamepad_events.iter() {
		match gamepad_event {
			GamepadEvent::Connection(connection_event) => {
				if connection_event.connected() {
					input.input_type = InputType::Gamepad;
					return;
				} else {
					if gamepads.iter().count() == 0 {
						input.input_type = InputType::Keyboard;
						return;
					}
				}
			}
			_ => {
				input.input_type = InputType::Gamepad;
				return;
			}
		}
	}
	if keyboard_input_events.iter().count() > 0 {
		input.input_type = InputType::Keyboard;
	}
}

pub fn sys_player_input(
	mut input: ResMut<PlayerInput>,
	keyboard: Res<Keyboard>,
	mouse: Res<Mouse>,
	q_camera: Query<(&Camera, &GlobalTransform), With<Camera>>,
	q_window: Query<&Window, With<PrimaryWindow>>,
	gamepads: Res<Gamepads>,
	gamepad: Res<Gamepad>,
	axes: Res<Axis<GamepadAxis>>,
) {
	match input.input_type {
		InputType::Keyboard => keyboard_input(&mut input, keyboard, mouse, q_camera, q_window),
		InputType::Gamepad => gamepad_input(&mut input, gamepads, gamepad, axes),
	}
}

fn keyboard_input(
	input: &mut ResMut<PlayerInput>,
	keyboard: Res<Keyboard>,
	mouse: Res<Mouse>,
	q_camera: Query<(&Camera, &GlobalTransform), With<Camera>>,
	q_window: Query<&Window, With<PrimaryWindow>>,
) {
	let keys = &keyboard.keys;

	// direction

	let mut dir = Vec2::ZERO;
	if keys.pressed(KeyCode::W) {
		dir.y += 1.0;
	}
	if keys.pressed(KeyCode::S) {
		dir.y -= 1.0;
	}
	if keys.pressed(KeyCode::A) {
		dir.x -= 1.0;
	}
	if keys.pressed(KeyCode::D) {
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

	input.primary = mouse.buttons.pressed(MouseButton::Left);

	if keys.just_released(KeyCode::F11) {
		input.full_screen = !input.full_screen;
	}

	if keys.just_released(KeyCode::F12) {
		input.debug = !input.debug;
	}
}

fn gamepad_input(
	input: &mut ResMut<PlayerInput>,
	gamepads: Res<Gamepads>,
	gamepad: Res<Gamepad>,
	axes: Res<Axis<GamepadAxis>>,
) {
	for id in gamepads.iter() {

		// direction

		let mut dir = Vec2::ZERO;
		let btn_left_x = GamepadAxis::new(id, GamepadAxisType::LeftStickX);
		if let Some(left_stick_x) = axes.get(btn_left_x) {
			dir.x = left_stick_x;
		}
		let btn_left_y = GamepadAxis::new(id, GamepadAxisType::LeftStickY);
		if let Some(left_stick_y) = axes.get(btn_left_y) {
			dir.y = left_stick_y;
		}
		input.dir = dir;

		// face

		let mut face = Vec2::ZERO;
		let btn_right_x = GamepadAxis::new(id, GamepadAxisType::RightStickX);
		if let Some(right_stick_x) = axes.get(btn_right_x) {
			face.x = right_stick_x;
		}
		let btn_right_y = GamepadAxis::new(id, GamepadAxisType::RightStickY);
		if let Some(right_stick_y) = axes.get(btn_right_y) {
			face.y = right_stick_y;
		}
		if face != Vec2::ZERO {
			input.face_turns = Vec2::X.angle_between(face) / TAU;
		}

		// action

		let btn_rt2 = GamepadButton::new(id, GamepadButtonType::RightTrigger2);
		if let Some(right_trigger) = gamepad.axis.get(btn_rt2) {
			input.primary = right_trigger.abs() >= 0.05;
		}

		// misc

		if gamepad.buttons.just_pressed(GamepadButton::new(id, GamepadButtonType::North)){
			input.full_screen = !input.full_screen
		}
		if gamepad.buttons.just_pressed(GamepadButton::new(id, GamepadButtonType::West)) {
			input.debug = !input.debug;
		}
	}
}
