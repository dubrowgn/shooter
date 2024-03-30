use bevy::{
	ecs::schedule::SystemConfigs,
	input::{
		ButtonState,
		gamepad::{GamepadButton, GamepadButtonChangedEvent, GamepadSettings},
		keyboard::KeyboardInput,
		mouse::MouseButtonInput,
	},
	prelude::*,
};

#[derive(Default, Resource, Reflect)]
struct InputEvents {
	pub consumed: bool,
}

#[derive(Default, Resource, Reflect)]
pub struct Keyboard {
	pub keys: Input<KeyCode>,
}

#[derive(Default, Resource, Reflect)]
pub struct Mouse {
	pub buttons: Input<MouseButton>,
}

#[derive(Default, Resource)]
pub struct Gamepad {
	pub axis: Axis<GamepadButton>,
	pub buttons: Input<GamepadButton>,
}

fn sys_queue_clear_input_events(mut inputs: ResMut<InputEvents>) {
	inputs.consumed = true;
}

fn sys_clear_input_events(
	mut inputs: ResMut<InputEvents>,
	mut gamepad_events: ResMut<Events<GamepadButtonChangedEvent>>,
	mut keyboard_events: ResMut<Events<KeyboardInput>>,
	mut mouse_btn_events: ResMut<Events<MouseButtonInput>>,
) {
	if !inputs.consumed {
		return;
	}

	inputs.consumed = false;
	gamepad_events.clear();
	keyboard_events.clear();
	mouse_btn_events.clear();
}

// from bevy's keyboard_input_system()
pub fn sys_collect_keyboard_events(
	mut keyboard: ResMut<Keyboard>,
	mut events: EventReader<KeyboardInput>,
) {
	// Avoid clearing if it's not empty to ensure change detection is not triggered.
	keyboard.bypass_change_detection().keys.clear();
	for event in events.read() {
		if let Some(key_code) = event.key_code {
			match event.state {
				ButtonState::Pressed => keyboard.keys.press(key_code),
				ButtonState::Released => keyboard.keys.release(key_code),
			}
		}
	}
}

// from bevy's mouse_button_input_system()
pub fn sys_collect_mouse_events(
	mut mouse: ResMut<Mouse>,
	mut events: EventReader<MouseButtonInput>,
) {
	mouse.bypass_change_detection().buttons.clear();
	for event in events.read() {
		match event.state {
			ButtonState::Pressed => mouse.buttons.press(event.button),
			ButtonState::Released => mouse.buttons.release(event.button),
		}
	}
}

// from bevy's gamepad_button_event_system()
pub fn sys_collect_gamepad_events(
	mut gamepad: ResMut<Gamepad>,
	mut events: EventReader<GamepadButtonChangedEvent>,
	settings: Res<GamepadSettings>,
) {
	gamepad.bypass_change_detection().buttons.clear();
	for event in events.read() {
		let btn = GamepadButton::new(event.gamepad, event.button_type);
		let prop = settings.get_button_settings(btn);

		// TODO: https://github.com/bevyengine/bevy/commit/60bbfd78acda269112039658998b68183a98ed0f
		// ~bevy 0.13?
		if event.value <= prop.release_threshold() {
			gamepad.buttons.release(btn);
		} else if event.value >= prop.press_threshold() {
			gamepad.buttons.press(btn);
		}

		gamepad.axis.set(btn, event.value);
	}
}

pub fn init(app: &mut App) -> &mut App {
	app
		.init_resource::<Events<GamepadButtonChangedEvent>>()
		.init_resource::<Events<KeyboardInput>>()
		.init_resource::<Events<MouseButtonInput>>()
		.init_resource::<InputEvents>()
		.init_resource::<Gamepad>()
		.init_resource::<Keyboard>()
		.init_resource::<Mouse>()
}

pub fn register(app: &mut App) -> &mut App {
	app
		.register_type::<InputEvents>()
		.register_type::<Keyboard>()
		.register_type::<Mouse>()
}

pub fn setup(app: &mut App) -> &mut App {
	register(app);
	init(app);
	app
}

pub fn systems_tick_input_collect() -> SystemConfigs {
	(
		sys_collect_gamepad_events,
		sys_collect_keyboard_events,
		sys_collect_mouse_events,
		sys_clear_input_events,
	).into_configs()
}

pub fn systems_input_gc() -> SystemConfigs {
	// theoretically, we want to clear events in Last (end of Update), since other
	// systems will want to consume input outside of the main client-tick schedule
	(
		sys_clear_input_events,
	).into_configs()
}

pub trait TickInputApp {
	fn init_tick_input(&mut self) -> &mut Self;
	fn register_tick_input(&mut self) -> &mut Self;
}

impl TickInputApp for App {
	fn init_tick_input(&mut self) -> &mut Self { init(self) }
	fn register_tick_input(&mut self) -> &mut Self { register(self) }
}
