use std::f32::consts::TAU;

use bevy::input::keyboard::KeyboardInput;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy::input::gamepad::{GamepadButton, GamepadEvent};

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
	input: ResMut<PlayerInput>,
	keyboard: Res<Input<KeyCode>>,
	mouse: Res<Input<MouseButton>>,
	q_camera: Query<(&Camera, &GlobalTransform), With<Camera>>,
	q_window: Query<&Window, With<PrimaryWindow>>,
	gamepads: Res<Gamepads>,
	button_inputs: Res<Input<GamepadButton>>,
	button_axes: Res<Axis<GamepadButton>>,
	axes: Res<Axis<GamepadAxis>>,
) {
	match input.input_type {
		InputType::Keyboard => keyboard_input(input, keyboard, mouse, q_camera, q_window),
		InputType::Gamepad => gamepad_input(input, gamepads, button_inputs, button_axes, axes),
	}
}

fn keyboard_input(
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

fn gamepad_input(
	mut input: ResMut<PlayerInput>,
	gamepads: Res<Gamepads>,
	button_inputs: Res<Input<GamepadButton>>,
	button_axes: Res<Axis<GamepadButton>>,
	axes: Res<Axis<GamepadAxis>>,
) {
	for gamepad in gamepads.iter() {

		// direction

		let left_stick_x = axes
			.get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickX))
			.unwrap();
		let left_stick_y = axes
			.get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickY))
			.unwrap();
		let dir = Vec2::new(left_stick_x, left_stick_y);
		input.dir = dir;

		// face

		let right_stick_x = axes
			.get(GamepadAxis::new(gamepad, GamepadAxisType::RightStickX))
			.unwrap();
		let right_stick_y = axes
			.get(GamepadAxis::new(gamepad, GamepadAxisType::RightStickY))
			.unwrap();
		if right_stick_x != 0.0 && right_stick_y != 0.0 {
			let delta = Vec2::new(right_stick_x, right_stick_y);
			input.face_turns = Vec2::X.angle_between(delta) / TAU;
		}

		// action

		let right_trigger = button_axes
			.get(GamepadButton::new(
				gamepad,
				GamepadButtonType::RightTrigger2,
			))
			.unwrap();
		input.primary = right_trigger.abs() >= 0.05;

		// misc

		if button_inputs.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::North)){
			input.full_screen = !input.full_screen
		}
		if button_inputs.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::West)) {
			input.debug = !input.debug;
		}
	}
}
