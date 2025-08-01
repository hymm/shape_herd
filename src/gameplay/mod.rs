mod enemy;
mod path;
mod physics;
mod player;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        enemy::EnemyPlugin,
        player::PlayerPlugin,
        path::PathPlugin,
        physics::PhysicsPlugin,
    ));
}
