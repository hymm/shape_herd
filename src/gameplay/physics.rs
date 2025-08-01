use bevy::{prelude::*, window::PrimaryWindow};

pub(crate) struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (apply_acceleration, apply_velocity, apply_screen_wrap).chain(),
        );
    }
}

#[derive(Component, Deref, DerefMut, Default)]
pub(crate) struct Acceleration(Vec2);

#[derive(Component, Deref, DerefMut, Default)]
pub(crate) struct Velocity(pub Vec2);

fn apply_velocity(mut q: Query<(&Velocity, &mut Transform)>, time: Res<Time>) {
    for (v, mut t) in &mut q {
        t.translation += v.extend(0.0) * time.delta_secs();
    }
}

fn apply_acceleration(mut q: Query<(&Acceleration, &mut Velocity)>, time: Res<Time>) {
    for (a, mut v) in &mut q {
        v.0 += a.0 * time.delta_secs();
    }
}

fn apply_screen_wrap(
    window: Single<&Window, With<PrimaryWindow>>,
    mut wrap_query: Query<&mut Transform>,
) {
    let size = window.size();
    let half_size = size / 2.0;
    for mut transform in &mut wrap_query {
        let position = transform.translation.xy();
        let wrapped = (position + half_size).rem_euclid(size) - half_size;
        transform.translation = wrapped.extend(transform.translation.z);
    }
}
