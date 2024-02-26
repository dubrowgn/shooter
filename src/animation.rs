use bevy::prelude::*;

#[derive(Component)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
	pub direction: isize,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

pub fn sys_animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &mut AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (mut indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            if sprite.index == indices.last {
                indices.direction = -1
            } else if sprite.index == indices.first {
                indices.direction = 1
            };
			sprite.index = sprite.index.wrapping_add_signed(indices.direction)
        }
    }
}