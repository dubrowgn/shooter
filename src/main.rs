use bevy::{
	input::Input,
	prelude::*,
	utils::{Duration, HashMap},
};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_inspector_egui_rapier::InspectableRapierPlugin;
use bevy_rapier2d::prelude::*;

const HALF_TURN: f32 = std::f32::consts::PI;
const QUARTER_TURN: f32 = HALF_TURN / 2.0;

const LAYER_BG: f32 = 1.0;
const LAYER_PLAYER: f32 = 2.0;

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
		.add_startup_system(load_assets)
		.add_startup_system(spawn_camera)
		.add_startup_system(spawn_player.after(load_assets))
		.add_startup_system(spawn_walls.after(load_assets))
		// systems
		.add_system(handle_input)
		.add_system(spawn_shot
			.after(handle_input)
			.before(update_camera)
		)
		.add_system(update_camera)
		// run
		.run();
}

type Textures = HashMap<String, Handle<TextureAtlas>>;

fn spawn_camera(mut cmds: Commands) {
	let scale = 1.5;
	let far  = 1000.0;

	let cam = Camera2dBundle {
		transform: Transform::from_xyz(0.0, 0.0, far)
			.with_scale(Vec3::new(scale, scale, 1.0)),
		..default()
	};

	cmds.spawn_bundle(cam);
}

fn update_camera(
	mut q_camera: Query<&mut Transform, With<Camera>>,
	q_player: Query<&Transform, (With<Player>, Without<Camera>)>
) {
	let p = q_player.single().translation;
	let c = &mut q_camera.single_mut().translation;
	c.x = p.x;
	c.y = p.y;
}

#[derive(Component, Default, Reflect)]
struct Player {
	shot_timer: Timer,
}

impl Player {
	fn new() -> Self {
		let mut p = Player {
			shot_timer: Timer::new(Duration::from_millis(100), true),
		};

		p.shot_timer.pause();

		return p;
	}
}

fn spawn_player(mut cmds: Commands, textures: Res<Textures>) {
	cmds.spawn()
		.insert(Player::new())
		.insert(Name::new("Player"))
		.insert_bundle(SpriteSheetBundle {
			texture_atlas: textures.get("player_purple").unwrap().clone(),
			transform: Transform::from_xyz(0.0, 0.0, LAYER_PLAYER),
			..default()
		})
		.insert(RigidBody::Dynamic)
		.insert(Collider::ball(96.0))
		.insert(Velocity::linear(Vec2::ZERO))
		.insert(Friction {
			coefficient: 0.0,
			combine_rule: CoefficientCombineRule::Min,
		});
}

#[derive(Component, Default, Reflect)]
struct Shot;

fn spawn_shot(
	mut cmds: Commands,
	textures: Res<Textures>,
	time: Res<Time>,
	mut q_player: Query<(&Transform, &mut Player)>
) {
	let (player_t, mut player) = q_player.single_mut();

	player.shot_timer.tick(time.delta());
	let shots = player.shot_timer.times_finished_this_tick();
	if shots < 1 {
		return;
	}

	let dir = player_t.right().truncate();
	let pos = player_t.translation.truncate() + dir * 96.0 + 26.0;
	let speed: f32 = 2700.0;

	for _ in 0..shots {
		cmds.spawn()
			.insert(Shot)
			.insert(Name::new("Shot"))
			.insert_bundle(SpriteSheetBundle {
				texture_atlas: textures.get("shot_purple").unwrap().clone(),
				transform: Transform::from_xyz(pos.x, pos.y, LAYER_PLAYER),
				..default()
			})
			.insert(RigidBody::Dynamic)
			.insert(Collider::ball(26.0))
			.insert(Velocity::linear(dir * speed))
			.insert(Restitution::coefficient(1.0))
			.insert(Friction {
				coefficient: 0.0,
				combine_rule: CoefficientCombineRule::Min,
			});
	}
}

#[derive(Component, Default, Reflect)]
struct Wall;

fn spawn_walls(mut cmds: Commands, textures: Res<Textures>) {
	cmds.spawn()
		.insert(Wall)
		.insert(Name::new("Wall - Left"))
		.insert_bundle(SpriteSheetBundle {
			texture_atlas: textures.get("wall_out_left").unwrap().clone(),
			transform: Transform::from_xyz(-1184.0, 0.0, LAYER_BG),
			..default()
		})
		.insert(RigidBody::Fixed)
		.insert(Collider::cuboid(96.0, 1920.0));

	cmds.spawn()
		.insert(Wall)
		.insert(Name::new("Wall - Right"))
		.insert_bundle(SpriteSheetBundle {
			texture_atlas: textures.get("wall_out_right").unwrap().clone(),
			transform: Transform::from_xyz(1184.0, 0.0, LAYER_BG),
			..default()
		})
		.insert(RigidBody::Fixed)
		.insert(Collider::cuboid(96.0, 1920.0));

	cmds.spawn()
		.insert(Wall)
		.insert(Name::new("Wall - Top"))
		.insert_bundle(SpriteSheetBundle {
			texture_atlas: textures.get("wall_out_top").unwrap().clone(),
			transform: Transform
				::from_rotation(Quat::from_rotation_z(QUARTER_TURN))
				.with_translation(Vec3::new(0.0, 1824.0, LAYER_BG)),
			..default()
		})
		.insert(RigidBody::Fixed)
		.insert(Collider::cuboid(96.0, 1280.0));

	cmds.spawn()
		.insert(Wall)
		.insert(Name::new("Wall - Bottom"))
		.insert_bundle(SpriteSheetBundle {
			texture_atlas: textures.get("wall_out_bottom").unwrap().clone(),
			transform: Transform
				::from_rotation(Quat::from_rotation_z(QUARTER_TURN))
				.with_translation(Vec3::new(0.0, -1824.0, LAYER_BG)),
			..default()
		})
		.insert(RigidBody::Fixed)
		.insert(Collider::cuboid(96.0, 1280.0));

	cmds.spawn()
		.insert(Wall)
		.insert(Name::new("Wall - Horizontal"))
		.insert_bundle(SpriteSheetBundle {
			texture_atlas: textures.get("wall_in_horizontal").unwrap().clone(),
			transform: Transform
				::from_rotation(Quat::from_rotation_z(QUARTER_TURN))
				.with_translation(Vec3::new(-196.0, -1149.5, LAYER_BG)),
			..default()
		})
		.insert(RigidBody::Fixed)
		.insert(Collider::cuboid(149.5, 533.0));

	cmds.spawn()
		.insert(Wall)
		.insert(Name::new("Wall - Verticle"))
		.insert_bundle(SpriteSheetBundle {
			texture_atlas: textures.get("wall_in_verticle").unwrap().clone(),
			transform: Transform::from_xyz(702.0, 288.5, LAYER_BG),
			..default()
		})
		.insert(RigidBody::Fixed)
		.insert(Collider::cuboid(148.0, 1232.5));
}

fn load_assets(
	mut textures: ResMut<Textures>,
	assets: Res<AssetServer>,
	mut atlases: ResMut<Assets<TextureAtlas>>
) {
	{
		let img: Handle<Image> = assets.load("image/players.png");
		let size = 256.0;
		let slice = |row: u16| TextureAtlas::from_grid_with_padding(
			img.clone(),
			Vec2::splat(size),
			3,
			1,
			Vec2::ZERO,
			Vec2::new(0.0, row as f32 * size),
		);
		textures.insert("player_blue".into(), atlases.add(slice(0)));
		textures.insert("player_red".into(), atlases.add(slice(1)));
		textures.insert("player_green".into(), atlases.add(slice(2)));
		textures.insert("player_purple".into(), atlases.add(slice(3)));
	}

	{
		let img: Handle<Image> = assets.load("image/shots.png");
		let size = 96.0;
		let slice = |row: u16| TextureAtlas::from_grid_with_padding(
			img.clone(),
			Vec2::splat(size),
			3,
			1,
			Vec2::ZERO,
			Vec2::new(0.0, row as f32 * size),
		);
		textures.insert("shot_blue".into(), atlases.add(slice(0)));
		textures.insert("shot_red".into(), atlases.add(slice(1)));
		textures.insert("shot_green".into(), atlases.add(slice(2)));
		textures.insert("shot_purple".into(), atlases.add(slice(3)));
	}

	{
		let img: Handle<Image> = assets.load("image/walls.png");
		let rect = |l, t, w, h| TextureAtlas::from_grid_with_padding(
			img.clone(),
			Vec2::new(w, h),
			1,
			1,
			Vec2::ZERO,
			Vec2::new(l, t),
		);
		textures.insert("wall_out_left".into(), atlases.add(rect(0.0, 0.0, 192.0, 3840.0)));
		textures.insert("wall_out_right".into(), atlases.add(rect(194.0, 0.0, 192.0, 3840.0)));
		textures.insert("wall_out_top".into(), atlases.add(rect(686.0, 0.0, 192.0, 2176.0)));
		textures.insert("wall_out_bottom".into(), atlases.add(rect(880.0, 0.0, 192.0, 2176.0)));
		textures.insert("wall_in_verticle".into(), atlases.add(rect(386.0, 0.0, 296.0, 2465.0)));
		textures.insert("wall_in_horizontal".into(), atlases.add(rect(386.0, 2465.0, 299.0, 1066.0)));
	}
}

fn handle_input(
	keys: Res<Input<KeyCode>>,
	btns: Res<Input<MouseButton>>,
	mut windows: ResMut<Windows>,
	q_camera: Query<(&Camera, &GlobalTransform), With<Camera>>,
	mut q_player: Query<(&mut Velocity, &mut Transform, &mut Player)>,
) {
	if keys.just_pressed(KeyCode::F11) {
		use bevy::window::WindowMode::{BorderlessFullscreen, Windowed};

		let win: &mut Window = windows.primary_mut();
		win.set_mode(if win.mode() == Windowed { BorderlessFullscreen } else { Windowed });
	}

	let (cam, cam_t) = q_camera.single();
	let (mut player_v, mut player_t, mut player) = q_player.single_mut();

	// shooting?
	if btns.just_pressed(MouseButton::Left) {
		let dur = player.shot_timer.duration();
		player.shot_timer.reset();
		player.shot_timer.unpause();
		player.shot_timer.set_elapsed(dur);
	} else if btns.just_released(MouseButton::Left) {
		player.shot_timer.pause();
	}

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
	player_v.linvel = 900.0 * d;
	player_v.angvel = 0.0;

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
