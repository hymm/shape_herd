use std::f32::consts::PI;

use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    gameplay::{
        path::DrawPath,
        physics::{Acceleration, Velocity},
    },
    screens::Screen,
};

pub(crate) struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Gameplay), spawn_player)
            .add_systems(
                Update,
                (point_player, accelerate_player, control_drawing)
                    .run_if(in_state(Screen::Gameplay)),
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
        DrawPath::default(),
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
    let Some(position) = cursor_pos.read().last() else {
        return Ok(());
    };

    let Ok(transform) = camera.single() else {
        return Ok(());
    };

    // convert cursor_pos into world coordinates
    let size = Vec2::new(window.width() as f32, window.height() as f32);
    let center_offset = (position.position - size / 2.0) * Vec2::new(1.0, -1.0);
    let world_cursor = center_offset + transform.translation.truncate();

    let mut player_transform = player_query.single_mut()?;

    let forward = Vec2::normalize(world_cursor - player_transform.translation.truncate());
    let angle = Vec2::Y.angle_to(forward);
    player_transform.rotation = Quat::from_rotation_z(angle);

    Ok(())
}

fn accelerate_player(
    button: Res<ButtonInput<KeyCode>>,
    player: Single<(&Transform, &Velocity, &mut Acceleration), With<Player>>,
) {
    // TODO: move these to a config file
    const FRICTION_TRANSVERSE: f32 = 200.0;
    const FRICTION_NEUTRAL: f32 = 50.0;
    const FRICTION_REVERSE: f32 = 75.0;
    const FORWARD_ACCEL: f32 = 100.0;

    let (t, v, mut a) = player.into_inner();

    // calculate forward acceleration
    let (_, _, angle) = t.rotation.to_euler(EulerRot::XYZ);
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

fn control_drawing(
    mut command: Commands,
    buttons: Res<ButtonInput<KeyCode>>,
    player: Single<&mut DrawPath, With<Player>>,
) {
    let mut draw = player.into_inner();
    match (buttons.just_pressed(KeyCode::Space), draw.active()) {
        (true, false) => draw.activate(),
        (true, true) => {
            if let Some(path_entity) = draw.path() {
                command.entity(path_entity).despawn();
            }
            draw.deactivate();
        }
        _ => {}
    }
}
