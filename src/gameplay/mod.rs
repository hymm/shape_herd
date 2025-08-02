mod enemy;
mod path;
mod physics;
mod player;
use avian2d::prelude::{Collider, RigidBody};
use bevy::{prelude::*, window::PrimaryWindow};

use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        enemy::EnemyPlugin,
        player::PlayerPlugin,
        path::PathPlugin,
        physics::PhysicsPlugin,
    ))
    .add_systems(OnEnter(Screen::Gameplay), spawn_window_colliders);
}

fn spawn_window_colliders(mut commands: Commands, window: Single<&Window, With<PrimaryWindow>>) {
    let size = window.size();
    // top
    commands.spawn((
        Collider::half_space(Vec2::NEG_Y),
        RigidBody::Kinematic,
        Transform::from_xyz(0.0, size.y / 2., 0.0),
    ));
    // bottom
    commands.spawn((
        Collider::half_space(Vec2::Y),
        RigidBody::Kinematic,
        Transform::from_xyz(0.0, -size.y / 2., 0.0),
    ));
    // right
    commands.spawn((
        Collider::half_space(Vec2::NEG_X),
        RigidBody::Kinematic,
        Transform::from_xyz(size.x / 2., 0.0, 0.0),
    ));
    // left
    commands.spawn((
        Collider::half_space(Vec2::X),
        RigidBody::Kinematic,
        Transform::from_xyz(-size.x / 2., 0.0, 0.0),
    ));
}
