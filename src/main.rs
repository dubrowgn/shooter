#![allow(dead_code)]

#[macro_use]
mod macros;

mod args;
mod collide;
mod debug;
mod layer;
mod metric;
mod movement;
mod tick_schedule;
mod time;

use args::parse_args;
use bevy::{
	input::Input,
	prelude::*,
	utils::{Duration, HashMap}, window::PrimaryWindow,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_prototype_lyon::plugin::ShapePlugin;
use collide::{
	Collidable,
	EntityHandle,
	sys_collide_debug_add,
	sys_collide_debug_toggle,
	toi,
	ToiResult,
};
use debug::{debug_enabled, Debug};
use layer::Layer;
use metric::Metric;
use movement::{Position, sys_write_back, Velocity};
use parry2d::partitioning::Qbvh;
use tick_schedule::{TickInfo, TickPlugin, TickSchedule};
use time::Accumulator;

const HALF_TURN: f32 = std::f32::consts::PI;
const QUARTER_TURN: f32 = HALF_TURN / 2.0;

fn main() {
	let config = unwrap!(parse_args(), {
		return;
	});

	println!("{:?}", config);

	let mut app = App::new();

	app
		// types
		.register_type::<Accumulator>()
		.register_type::<Player>()
		.register_type::<Position>()
		.register_type::<Shot>()
		.register_type::<Velocity>()

		// resources
		.insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
		.insert_resource(Debug { enabled: false })
		.insert_resource(Sounds(HashMap::new()))
		.insert_resource(Textures(HashMap::new()))
		.insert_resource(Statics(Qbvh::new()))

		// plugins
		.add_plugins(DefaultPlugins)
		.add_plugin(ShapePlugin)
		.add_plugin(WorldInspectorPlugin::default().run_if(debug_enabled))

		// startup systems
		.add_startup_systems((
			sys_window_setup,
			load_assets,
			spawn_camera,
			spawn_player.after(load_assets),
			spawn_bg.after(load_assets),
			spawn_statics.after(load_assets),
			sys_spawn_shots.after(load_assets),
		).in_base_set(StartupSet::Startup))
		.add_startup_systems((
			sys_index_statics,
		).in_base_set(StartupSet::PostStartup))

		// tick
		.insert_resource(TickInfo {
			acc: Duration::ZERO,
			budget: Duration::from_millis(100),
			step: Duration::from_nanos(1_000_000_000 / 60),
		})
		.add_plugin(TickPlugin)

		.add_systems((
			handle_input,
		).in_schedule(TickSchedule::PreTicks))
		.add_systems((
			sys_tps,
			sys_spawn_shot,
			sys_move_shots.after(sys_spawn_shot),
			sys_move_player.after(sys_move_shots),
		).in_schedule(TickSchedule::Ticks))
		.add_systems((
			sys_fps,
			sys_write_back,
			update_camera,
		).in_schedule(TickSchedule::PostTicks))

		// systems
		.add_system(sys_collide_debug_add)
		.add_system(sys_collide_debug_toggle)
		;

	app
		// run
		.run();
}

#[derive(Resource)]
struct Sounds(HashMap<String, Handle<AudioSource>>);

#[derive(Resource)]
struct Textures(HashMap<String, Handle<TextureAtlas>>);

#[derive(Resource)]
struct Statics(Qbvh<EntityHandle>);

fn sys_window_setup(mut window: Query<&mut Window>) {
	window.single_mut().title = "shooter".into();
}

fn spawn_camera(mut cmds: Commands) {
	let scale = 1.5;
	let far  = 1000.0;

	cmds.spawn(Camera2dBundle {
		transform: Transform::from_xyz(0.0, 0.0, far)
			.with_scale(Vec3::new(scale, scale, 1.0)),
		..default()
	});
}

fn update_camera(
	mut q_camera: Query<&mut Transform, With<Camera>>,
	q_player: Query<&Position, (With<Player>, Without<Camera>)>
) {
	let pos = q_player.single();
	let c = &mut q_camera.single_mut().translation;
	c.x = pos.p.x;
	c.y = pos.p.y;
}

fn sys_index_statics(
	mut statics: ResMut<Statics>,
	q_walls: Query<(Entity, &Collidable, &Position), With<Static>>
) {
	let statics = &mut statics.0;
	let shapes = q_walls.iter()
		.map(|(ent, ref col, ref pos)| (EntityHandle::from(ent), col.shape.compute_aabb(&pos.to_iso())));
	statics.clear_and_rebuild(shapes, 0.0);
}

fn reflect(v: Vec2, norm: Vec2) -> Vec2 {
	v - 2.0 * v.dot(norm) * norm
}

fn slide(v: Vec2, norm: Vec2) -> Vec2 {
	v - v.dot(norm) * norm
}

fn sys_move_shots(
	mut cmds: Commands,
	statics: Res<Statics>,
	tick: Res<TickInfo>,
	mut q_shots: Query<(Entity, &Collidable, &mut Position, &mut Velocity, &mut Shot)>,
	q_statics: Query<(Entity, &Collidable, &Position), (With<Static>, Without<Shot>)>,
) {
	let statics = &statics.0;
	let step_secs = tick.step.as_secs_f32();

	for (ent, col, mut pos, mut vel, mut shot) in &mut q_shots {

		//info!("    ----");

		let mut max_toi = step_secs;
		let mut limit = 8;
		while max_toi > 0.0 && limit > 0 {
			limit -= 1;

			//info!("pos: {:?}; v: {:?}; max_toi: {}", pos.p, vel.v, max_toi);

			let margin:f32 = 1024.0 * f32::EPSILON;
			match toi(&q_statics, &statics, col, &pos, &vel, max_toi) {
				ToiResult::Miss => {
					pos.p += vel.v * max_toi;
					break;
				},
				ToiResult::Contact(contact) => {
					//info!("contact: {:?}", contact);

					pos.p += contact.norm * (contact.dist + margin);
				},
				ToiResult::Toi(toi) => {
					//info!("toi: {:?}", toi);

					if shot.bounces == 0 {
						cmds.entity(ent)
							.despawn_recursive();
						break;
					}

					shot.bounces -= 1;

					max_toi -= toi.toi_sec;
					pos.p += vel.v * toi.toi_sec + toi.norm * margin;
					vel.v = reflect(vel.v, toi.norm);
				},
			}
		}
	}
}

fn sys_move_player(
	statics: Res<Statics>,
	tick: Res<TickInfo>,
	mut q_player: Query<(&Collidable, &mut Position, &Velocity), With<Player>>,
	q_statics: Query<(Entity, &Collidable, &Position), (With<Static>, Without<Player>)>,
) {
	let statics = &statics.0;
	let step_secs = tick.step.as_secs_f32();

	for (col, mut pos, vel) in &mut q_player {
		if vel.v == Vec2::ZERO {
			continue;
		}

		//info!("    ----");

		let mut max_toi = step_secs;
		let mut v = vel.clone();
		let mut limit = 8;
		while max_toi > 0.0 && limit > 0 {
			limit -= 1;

			//info!("pos: {:?}; v: {:?}; max_toi: {}", pos.p, v.v, max_toi);

			let margin:f32 = 8192.0 * f32::EPSILON;
			match toi(&q_statics, &statics, col, &pos, &v, max_toi) {
				ToiResult::Miss => {
					pos.p += v.v * max_toi;
					break;
				},
				ToiResult::Contact(contact) => {
					//info!("contact: {:?}", contact);

					pos.p += contact.norm * (contact.dist + margin);
					v.v = slide(v.v, contact.norm);
				},
				ToiResult::Toi(toi) => {
					//info!("toi: {:?}", toi);

					max_toi -= toi.toi_sec;
					pos.p += v.v * toi.toi_sec + toi.norm * margin;
					v.v = slide(v.v, toi.norm);
				},
			}
		}
	}
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct Player {
	shot_acc: Option<Accumulator>,
}

fn spawn_player(mut cmds: Commands, textures: Res<Textures>) {
	let textures = &textures.0;

	cmds.spawn((
		Player::default(),
		Name::new("Player"),
		SpriteSheetBundle {
			texture_atlas: textures.get("player_purple").unwrap().clone(),
			transform: Transform::from_xyz(0.0, 0.0, Layer::PLAYER),
			..default()
		},
		Collidable::circle(96.0),
		Position::ZERO,
		Velocity::ZERO,
	));
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct Shot {
	bounces: u8,
}

fn spawn_shot(
	cmds: &mut Commands,
	textures: &Res<Textures>,
	pos: Vec2,
	dir: Vec2,
) {
	let textures = &textures.0;
	let speed: f32 = 2700.0;

	cmds.spawn((
		Shot{ bounces: 3 },
		Name::new("Shot"),
		SpriteSheetBundle {
			texture_atlas: textures.get("shot_purple").unwrap().clone(),
			transform: Transform::from_xyz(pos.x, pos.y, Layer::SHOT),
			..default()
		},
		Collidable::circle(26.0),
		Position::from(pos),
		Velocity::from(dir * speed),
	));
}

fn sys_spawn_shot(
	mut cmds: Commands,
	audio: Res<Audio>,
	sounds: Res<Sounds>,
	textures: Res<Textures>,
	tick: Res<TickInfo>,
	mut q_player: Query<(&Transform, &mut Player)>
) {
	let sounds = &sounds.0;
	let step_ns = tick.step.as_nanos();

	let (player_t, mut player) = q_player.single_mut();
	let dir = player_t.right().truncate();
	let pos = player_t.translation.truncate() + dir * (96.0 + 26.0);

	if let Some(acc) = &mut player.shot_acc {
		for _ in acc.advance(step_ns as u64) {
			spawn_shot(&mut cmds, &textures, pos, dir);
			audio.play_with_settings(
				sounds.get("laser/1").unwrap().clone(),
				PlaybackSettings {
					repeat: false,
					speed: 1.0,
					volume: 1.0,
				},
			);
		}
	}
}

fn sys_spawn_shots(
	mut cmds: Commands,
	textures: Res<Textures>,
) {
	let half_range = 0;
	for i in -half_range..half_range {
		let f = 0.001 * i as f32;
		let pos = Vec2::new(f, f);
		let dir = Vec2::from_angle(f);
		spawn_shot(&mut cmds, &textures, pos, dir);
	}
}

#[derive(Component)]
struct Bg;

fn spawn_bg(mut cmds: Commands, textures: Res<Textures>) {
	let textures = &textures.0;

	let mut mk_grass = |x, y| {
		cmds.spawn((
			Bg,
			Name::new(format!("Grass ({}, {})", x, y)),
			SpriteSheetBundle {
				texture_atlas: textures.get("grass").unwrap().clone(),
				transform: Transform::from_xyz(x, y, Layer::BG),
				..default()
			},
			Position::new(x, y),
		));
	};

	let xn = 8;
	let yn = 12;
	let size: f32 = 320.0;
	let left: f32 = -size / 2.0 - (xn / 2 - 1) as f32 * size;
	let bottom: f32 = -size / 2.0 - (yn / 2 - 1) as f32 * size;
	for xi in 0..xn {
		for yi in 0..yn {
			mk_grass(left + xi as f32 * size, bottom + yi as f32 * size);
		}
	}

	let mut mk_dirt = |x, y| {
		cmds.spawn((
			Bg,
			Name::new("Dirt"),
			SpriteSheetBundle {
				texture_atlas: textures.get("dirt").unwrap().clone(),
				transform: Transform::from_xyz(x, y, Layer::BG_FX),
				..default()
			},
			Position::new(x, y),
		));
	};

	mk_dirt(-260.0, 240.0);
}

#[derive(Component, Default, Reflect)]
struct Static;

fn spawn_statics(mut cmds: Commands, textures: Res<Textures>) {
	let textures = &textures.0;

	{
		let mut mk_wall = |name, texture, x, y, w, h, r| {
			cmds.spawn((
				Static,
				Name::new(name),
				SpriteSheetBundle {
					texture_atlas: textures.get(texture).unwrap().clone(),
					transform: Transform
						::from_rotation(Quat::from_rotation_z(r))
						.with_translation(Vec3::new(x, y, Layer::STATIC)),
					..default()
				},
				Collidable::aa_rect(w, h),
				Position::new(x, y),
			));
		};

		mk_wall("Wall - Left", "wall_out_left", -1184.0, 0.0, 96.0, 3840.0, 0.0);
		mk_wall("Wall - Right", "wall_out_right", 1184.0, 0.0, 96.0, 3840.0, 0.0);
		mk_wall("Wall - Top", "wall_out_top", 0.0, 1824.0, 2560.0, 96.0, QUARTER_TURN);
		mk_wall("Wall - Bottom", "wall_out_bottom", 0.0, -1824.0, 2560.0, 96.0, QUARTER_TURN);
		mk_wall("Wall - Horizontal", "wall_in_horizontal", -196.0, -1149.5, 1066.0, 299.0, QUARTER_TURN);
		mk_wall("Wall - Verticle", "wall_in_verticle", 702.0, 288.5, 296.0, 2465.0, 0.0);
	}

	{
		let mut mk_bush = |x, y| {
			cmds.spawn((
				Static,
				Name::new(format!("Bush ({}, {})", x, y)),
				SpriteSheetBundle {
					texture_atlas: textures.get("bush").unwrap().clone(),
					transform: Transform::from_xyz(x, y, Layer::STATIC),
					..default()
				},
				Collidable::circle(128.0),
				Position::new(x, y),
			));
		};

		mk_bush(-128.0, 1228.0);
		mk_bush(128.0, 1100.0);
		mk_bush(-512.0, 64.0);
		mk_bush(192.0, -512.0);
		mk_bush(64.0, -640.0);
		mk_bush(760.0, -1400.0);
	}
}

fn load_assets(
	mut sounds: ResMut<Sounds>,
	mut textures: ResMut<Textures>,
	assets: Res<AssetServer>,
	mut atlases: ResMut<Assets<TextureAtlas>>
) {
	let sounds = &mut sounds.0;
	let textures = &mut textures.0;

	let slice = |img: &Handle<Image>, size, row: u16|
		TextureAtlas::from_grid(
			img.clone(),
			Vec2::splat(size),
			3,
			1,
			Some(Vec2::ZERO),
			Some(Vec2::new(0.0, row as f32 * size)),
		);

	{
		let img = assets.load("image/players.png");
		let size = 256.0;
		textures.insert("player_blue".into(), atlases.add(slice(&img, size, 0)));
		textures.insert("player_red".into(), atlases.add(slice(&img, size, 1)));
		textures.insert("player_green".into(), atlases.add(slice(&img, size, 2)));
		textures.insert("player_purple".into(), atlases.add(slice(&img, size, 3)));
	}

	{
		let img = assets.load("image/shots.png");
		let size = 96.0;
		textures.insert("shot_blue".into(), atlases.add(slice(&img, size, 0)));
		textures.insert("shot_red".into(), atlases.add(slice(&img, size, 1)));
		textures.insert("shot_green".into(), atlases.add(slice(&img, size, 2)));
		textures.insert("shot_purple".into(), atlases.add(slice(&img, size, 3)));
	}

	let rect = |img: &Handle<Image>, l, t, w, h|
		TextureAtlas::from_grid(
			img.clone(),
			Vec2::new(w, h),
			1,
			1,
			Some(Vec2::ZERO),
			Some(Vec2::new(l, t)),
		);

	{
		let img = assets.load("image/walls.png");
		textures.insert("wall_out_left".into(), atlases.add(rect(&img, 0.0, 0.0, 192.0, 3840.0)));
		textures.insert("wall_out_right".into(), atlases.add(rect(&img, 194.0, 0.0, 192.0, 3840.0)));
		textures.insert("wall_out_top".into(), atlases.add(rect(&img, 686.0, 0.0, 192.0, 2176.0)));
		textures.insert("wall_out_bottom".into(), atlases.add(rect(&img, 880.0, 0.0, 192.0, 2176.0)));
		textures.insert("wall_in_verticle".into(), atlases.add(rect(&img, 386.0, 0.0, 296.0, 2465.0)));
		textures.insert("wall_in_horizontal".into(), atlases.add(rect(&img, 386.0, 2465.0, 299.0, 1066.0)));
	}

	{
		let img = assets.load("image/bush.png");
		textures.insert("bush".into(), atlases.add(rect(&img, 0.0, 0.0, 256.0, 256.0)));
	}

	{
		let img = assets.load("image/grass.png");
		textures.insert("grass".into(), atlases.add(rect(&img, 1.0, 1.0, 320.0, 320.0)));
	}

	{
		let img = assets.load("image/dirt_splat.png");
		textures.insert("dirt".into(), atlases.add(rect(&img, 0.0, 0.0, 640.0, 640.0)));
	}

	sounds.insert("laser/1".into(), assets.load("audio/laser/1.ogg"));
	sounds.insert("laser/2".into(), assets.load("audio/laser/2.ogg"));
	sounds.insert("laser/3".into(), assets.load("audio/laser/3.ogg"));
	sounds.insert("laser/4".into(), assets.load("audio/laser/4.ogg"));
	sounds.insert("laser/5".into(), assets.load("audio/laser/5.ogg"));
}

fn sys_fps(mut metric: Local<Metric>, time: Res<Time>) {
	metric.sample(time.delta_seconds());
	if metric.total() >= 1.0 {
		info!(
			"frames:{}, fps:{:.2}, min:{:.2}ms, max:{:.2}ms",
			metric.count(),
			1.0 / metric.avg(),
			metric.min() * 1000.0,
			metric.max() * 1000.0,
		);
		metric.reset();
	}
}

fn sys_tps(mut metric: Local<Metric>, tick: Res<TickInfo>) {
	metric.sample(tick.step.as_secs_f32());
	if metric.total() >= 1.0 {
		info!(
			"ticks:{}, tps:{:.2}, min:{:.2}ms, max:{:.2}ms",
			metric.count(),
			1.0 / metric.avg(),
			metric.min() * 1000.0,
			metric.max() * 1000.0,
		);
		metric.reset();
	}
}

fn handle_input(
	keys: Res<Input<KeyCode>>,
	btns: Res<Input<MouseButton>>,
	mut debug: ResMut<Debug>,
	q_camera: Query<(&Camera, &GlobalTransform), With<Camera>>,
	mut q_player: Query<(&mut Velocity, &mut Transform, &mut Player)>,
	mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
) {
	let (cam, cam_t) = q_camera.single();
	let mut win = q_windows.single_mut();

	if keys.just_pressed(KeyCode::F11) {
		use bevy::window::WindowMode::{BorderlessFullscreen, Windowed};

		win.mode = if win.mode == Windowed { BorderlessFullscreen } else { Windowed };
	}

	if keys.just_released(KeyCode::F12) {
		debug.enabled = !debug.enabled;
	}

	let (mut player_v, mut player_t, mut player) = q_player.single_mut();

	// shooting?
	if btns.just_pressed(MouseButton::Left) {
		player.shot_acc = Some(Accumulator::ready_from_millis(100));
	} else if btns.just_released(MouseButton::Left) {
		player.shot_acc = None;
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
	player_v.v = 900.0f32 * d.normalize_or_zero();

	// rotation

	let win_pos = win.cursor_position();
	if win_pos == None {
		return;
	}

	let world_pos = win.cursor_position()
		.and_then(|win_pos| cam.viewport_to_world_2d(cam_t, win_pos));
	if  world_pos == None {
		return;
	}

	let delta_t = world_pos.unwrap() - player_t.translation.truncate();
	let rads = Vec2::X.angle_between(delta_t);

	player_t.rotation = Quat::from_rotation_z(rads);
}
