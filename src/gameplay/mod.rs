mod enemy;
mod enemy_ai;
mod layers;
mod map;
mod path;
mod physics;
mod player;
mod rng_bag;
mod score;
mod state;
use avian2d::prelude::{Collider, RigidBody};
use bevy::{prelude::*, window::PrimaryWindow};

use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        enemy::EnemyPlugin,
        enemy_ai::EnemyAiPlugin,
        player::PlayerPlugin,
        path::PathPlugin,
        physics::PhysicsPlugin,
        score::ScorePlugin,
        state::PlayingStatePlugin,
        map::MapPlugin,
    ))
    .add_systems(OnEnter(Screen::Gameplay), spawn_window_colliders);
}

#[derive(SystemSet, Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
struct DespawnSet;

#[derive(Component)]
struct Wall;

fn spawn_window_colliders(mut commands: Commands, window: Single<&Window, With<PrimaryWindow>>) {
    let size = window.size();
    // top
    commands.spawn((
        Collider::half_space(Vec2::NEG_Y),
        RigidBody::Kinematic,
        Transform::from_xyz(0.0, size.y / 2., 0.0),
        Wall,
    ));
    // bottom
    commands.spawn((
        Collider::half_space(Vec2::Y),
        RigidBody::Kinematic,
        Transform::from_xyz(0.0, -size.y / 2., 0.0),
        Wall,
    ));
    // right
    commands.spawn((
        Collider::half_space(Vec2::NEG_X),
        RigidBody::Kinematic,
        Transform::from_xyz(size.x / 2., 0.0, 0.0),
        Wall,
    ));
    // left
    commands.spawn((
        Collider::half_space(Vec2::X),
        RigidBody::Kinematic,
        Transform::from_xyz(-size.x / 2., 0.0, 0.0),
        Wall,
    ));
}
