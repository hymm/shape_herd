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
    /// scalar acceleration to apply in direction of player
    /// use negative acceleration to move away from player
    pub acceleration: f32,
    /// react to playter within this distance
    pub distance: f32,
}

fn follow_player(
    player: Single<&Transform, With<Player>>,
    mut followers: Query<(&mut ExternalForce, &Transform, &FollowPlayer)>,
) {
    for (mut f, t, follow) in &mut followers {
        let player_direction = (player.translation - t.translation).truncate();
        if player_direction.length() < follow.distance {
            **f = follow.acceleration * player_direction;
        } else {
            **f = Vec2::ZERO;
        }
    }
}
