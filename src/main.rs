use bevy::{
	input::Input,
	prelude::*,
	utils::HashMap,
};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_inspector_egui_rapier::InspectableRapierPlugin;
use bevy_rapier2d::prelude::*;

fn main() {
	App::new()
		// types
		.register_type::<Player>()
		// resources
		.insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
		.insert_resource(WindowDescriptor {
			title: "shooter".to_string(),
			..default()
		})
		.insert_resource(Textures::new())
		.insert_resource(RapierConfiguration {
			gravity: Vec2::ZERO,
			..default()
		})
		// plugins
		.add_plugins(DefaultPlugins)
		.add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
		.add_plugin(RapierDebugRenderPlugin::default())
		.add_plugin(InspectableRapierPlugin)
		.add_plugin(WorldInspectorPlugin::new())
		// startup systems
		.add_startup_system_to_stage(StartupStage::PreStartup, load_assets)
		.add_startup_system(spawn_camera)
		.add_startup_system(spawn_player)
		// systems
		.add_system_to_stage(CoreStage::PreUpdate, handle_input)
		// run
		.run();
}

type Textures = HashMap<String, Handle<TextureAtlas>>;

fn spawn_camera(mut cmds: Commands) {
	let cam = Camera2dBundle::default();

	cmds.spawn_bundle(cam);
}

#[derive(Component, Default, Reflect)]
struct Player;

fn spawn_player(mut cmds: Commands, textures: Res<Textures>) {
	cmds.spawn()
		.insert(Player)
		.insert(Name::new("Player"))
		.insert_bundle(SpriteSheetBundle {
			texture_atlas: textures.get("player_blue").unwrap().clone(),
			..default()
		})
		.insert(RigidBody::KinematicVelocityBased)
		.insert(Collider::ball(100.0))
		.insert(Velocity::linear(Vec2::ZERO));
}

fn load_assets(
	mut textures: ResMut<Textures>,
	assets: Res<AssetServer>,
	mut atlases: ResMut<Assets<TextureAtlas>>
) {
	let img: Handle<Image> = assets.load("image/player_blue.png");
	let handle = atlases.add(TextureAtlas::from_grid(
		img,
		Vec2::splat(256.0),
		3,
		1
	));

	textures.insert("player_blue".to_string(), handle.clone());
}

fn handle_input(
	keys: Res<Input<KeyCode>>,
	windows: Res<Windows>,
	q_camera: Query<(&Camera, &GlobalTransform), With<Camera>>,
	mut q_player: Query<(&mut Velocity, &mut Transform), With<Player>>,
) {
	let (cam, cam_t) = q_camera.single();
	let (mut player_v, mut player_t) = q_player.single_mut();

	// velocity

	let mut d = Vec2::ZERO;
	if keys.pressed(KeyCode::W) {
		d.y += 1.0;
	}
	if keys.pressed(KeyCode::S) {
		d.y -= 1.0;
	}
	if keys.pressed(KeyCode::A) {
		d.x -= 1.0;
	}
	if keys.pressed(KeyCode::D) {
		d.x += 1.0;
	}
	player_v.linvel = 512.0 * d;

	// rotation

	let win = windows
		.get_primary()
		.unwrap();
	let win_pos = win.cursor_position();
	if win_pos == None {
		return;
	}

	let win_size = Vec2::new(win.width() as f32, win.height() as f32);

	// convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
	let ndc = win_pos.unwrap() / win_size * 2.0 - Vec2::ONE;

	// matrix for undoing the projection and camera transform
	let ndc_to_world = cam_t.compute_matrix() * cam.projection_matrix().inverse();

	// use it to convert ndc to world-space coordinates
	let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

	let delta_t = world_pos - player_t.translation;
	let rads = Vec2::X.angle_between(delta_t.truncate());

	player_t.rotation = Quat::from_rotation_z(rads);
}
