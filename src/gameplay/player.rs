use std::f32::consts::PI;

use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    gameplay::physics::{Acceleration, Velocity},
    screens::Screen,
};

pub(crate) struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Gameplay), spawn_player)
            .add_systems(
                Update,
                (point_player, accelerate_player).run_if(in_state(Screen::Gameplay)),
            );
    }
}

#[derive(Component)]
struct Player;

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh_handle = meshes.add(Triangle2d::new(
        Vec2::Y * 8.0,
        Vec2::new(-5.0, -8.0),
        Vec2::new(5.0, -8.0),
    ));
    let mat_handle = materials.add(Color::hsl(0., 1.0, 1.0));
    commands.spawn((
        Player,
        Name::new("Player"),
        Mesh2d(mesh_handle),
        MeshMaterial2d(mat_handle),
        Velocity::default(),
        Acceleration::default(),
    ));
}

fn point_player(
    // TODO: can we get the off window coordinates?
    mut cursor_pos: EventReader<CursorMoved>,
    camera: Query<&Transform, (With<Camera>, Without<Player>)>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut player_query: Query<&mut Transform, With<Player>>,
) -> Result<(), BevyError> {
    let window = window.single()?;
    let mut player_point = None;
    for position in cursor_pos.read() {
        let Ok(transform) = camera.single() else {
            continue;
        };
        // convert cursor_pos into world coordinates
        let size = Vec2::new(window.width() as f32, window.height() as f32);
        player_point = Some(position.position - size / 2.0 + transform.translation.truncate());
    }

    let Some(target) = player_point else {
        // nothing to do
        return Ok(());
    };

    let mut player_transform = player_query.single_mut()?;

    let forward = Vec2::normalize(target - player_transform.translation.truncate());
    let angle = Vec2::Y.angle_to(forward);
    // we need to negate the angle since neg z is direction of camera.(???)
    player_transform.rotation = Quat::from_rotation_z(PI - angle);

    Ok(())
}

fn accelerate_player(
    button: Res<ButtonInput<KeyCode>>,
    player: Single<(&Transform, &Velocity, &mut Acceleration), With<Player>>,
) {
    // TODO: move these to a config file
    const FRICTION_TRANSVERSE: f32 = 10.0;
    const FRICTION_NEUTRAL: f32 = 10.0;
    const FRICTION_REVERSE: f32 = 50.0;
    const FORWARD_ACCEL: f32 = 100.0;

    let (t, v, mut a) = player.into_inner();

    // calculate forward acceleration
    let (_axis, angle) = t.rotation.to_axis_angle();
    let forward = Vec2::from_angle(angle + PI / 2.);
    let v_forward = v.dot(forward);
    let a_forward = if button.pressed(KeyCode::KeyW) {
        FORWARD_ACCEL
    } else if button.pressed(KeyCode::KeyS) {
        -FRICTION_REVERSE
    } else if v_forward < 0.0 {
        FRICTION_NEUTRAL
    } else if v_forward > 0.0 {
        -FRICTION_NEUTRAL
    } else {
        0.0
    };

    // calculate transverse acceleration
    let transverse = forward.perp();
    let v_transverse = v.dot(transverse);
    let a_transverse = if v_transverse > 0.0 {
        -FRICTION_TRANSVERSE
    } else if v_transverse < 0.0 {
        FRICTION_TRANSVERSE
    } else {
        0.0
    };

    **a = forward.normalize() * a_forward + transverse.normalize() * a_transverse;
}
