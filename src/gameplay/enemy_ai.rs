use crate::gameplay::player::Player;
use avian2d::prelude::ExternalForce;
use bevy::prelude::*;

pub struct EnemyAiPlugin;
impl Plugin for EnemyAiPlugin {
    fn build(&self, app: &mut App) {
        // FixedUpdate runs before FixedPostUpdate which runs the physics schedule
        app.add_systems(FixedUpdate, follow_player);
    }
}

#[derive(Component)]
#[require(ExternalForce)]
pub struct FollowPlayer {
    pub active: bool,
    /// scalar acceleration to apply in direction of player
    pub acceleration: f32,
}

fn follow_player(
    player: Single<&Transform, With<Player>>,
    mut followers: Query<(&mut ExternalForce, &Transform, &FollowPlayer)>,
) {
    for (mut f, t, follow) in &mut followers {
        **f = follow.acceleration * (player.translation - t.translation).truncate()
    }
}
