use bevy::prelude::*;

/// Currently only controls player. other physics are implemented using avian
/// TODO: try using avian dynamic body for player
pub(crate) struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (apply_acceleration, apply_velocity).chain());
    }
}

#[derive(Component, Deref, DerefMut, Default)]
pub(crate) struct Acceleration(pub(crate) Vec2);

#[derive(Component, Deref, DerefMut, Default, Clone, Copy)]
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
