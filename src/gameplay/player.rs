use std::f32::consts::PI;

use avian2d::prelude::{
    CoefficientCombine, Collider, Collisions, Friction, Restitution, RigidBody,
};
use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    gameplay::{
        DespawnSet, Wall,
        enemy::EnemyType,
        path::DrawPath,
        physics::{Acceleration, MaxSpeed, Velocity},
        state::Playing,
    },
    screens::Screen,
};

pub(crate) struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Gameplay), spawn_player)
            .add_systems(
                Update,
                (
                    point_player,
                    accelerate_player,
                    (control_drawing, handle_player_collisions),
                )
                    .chain()
                    .run_if(in_state(Screen::Gameplay))
                    .run_if(in_state(Playing::Live)),
            )
            .add_systems(OnEnter(Playing::Dying), despawn_player)
            .add_systems(OnExit(Screen::Gameplay), despawn_player.in_set(DespawnSet));
    }
}

#[derive(Component)]
pub(crate) struct Player;

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let triangle_points = [Vec2::Y * 8.0, Vec2::new(-5.0, -8.0), Vec2::new(5.0, -8.0)];
    let mesh_handle = meshes.add(Triangle2d::new(
        triangle_points[0],
        triangle_points[1],
        triangle_points[2],
    ));
    let mat_handle = materials.add(Color::hsl(0., 1.0, 1.0));
    commands.spawn((
        Player,
        Name::new("Player"),
        Mesh2d(mesh_handle),
        MeshMaterial2d(mat_handle),
        MaxSpeed(300.0),
        Velocity::default(),
        Acceleration::default(),
        DrawPath::default(),
        RigidBody::Kinematic,
        Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
        Collider::triangle(triangle_points[0], triangle_points[1], triangle_points[2]),
        Restitution::new(0.8),
        Transform::from_xyz(0.0, 0.0, crate::gameplay::layers::ON_GROUND),
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
    let size = Vec2::new(window.width(), window.height());
    let center_offset = (position.position - size / 2.0) * Vec2::new(1.0, -1.0);
    let world_cursor = center_offset + transform.translation.truncate();

    let mut player_transform = player_query.single_mut()?;

    let forward = Vec2::normalize(world_cursor - player_transform.translation.truncate());
    let angle = Vec2::Y.angle_to(forward);
    player_transform.rotation = Quat::from_rotation_z(angle);

    Ok(())
}

fn accelerate_player(
    key: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    player: Single<(&Transform, &Velocity, &mut Acceleration), With<Player>>,
) {
    // TODO: move these to a config file
    const FRICTION_TRANSVERSE: f32 = 1000.0;
    const FRICTION_NEUTRAL: f32 = 50.0;
    const FRICTION_BRAKE: f32 = 2000.0;
    const FORWARD_ACCEL: f32 = 200.0;
    const FORWARD_ACCEL_REVERSE: f32 = 800.0;

    let (t, v, mut a) = player.into_inner();

    // calculate forward acceleration
    let (_, _, angle) = t.rotation.to_euler(EulerRot::XYZ);
    let forward = Vec2::from_angle(angle + PI / 2.);
    let v_forward = v.dot(forward);
    let a_forward = if key.pressed(KeyCode::KeyW) || mouse.pressed(MouseButton::Left) {
        if v_forward >= 0.0 {
            FORWARD_ACCEL
        } else {
            FORWARD_ACCEL_REVERSE
        }
    } else if key.pressed(KeyCode::KeyS) || mouse.pressed(MouseButton::Middle) {
        if v_forward > 0.0 {
            -FRICTION_BRAKE
        } else if v_forward < 0.0 {
            FRICTION_BRAKE
        } else {
            0.0
        }
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
    key: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    player: Single<&mut DrawPath, With<Player>>,
) {
    let mut draw = player.into_inner();
    match (
        key.just_pressed(KeyCode::Space) || mouse.just_pressed(MouseButton::Right),
        draw.active(),
    ) {
        (true, false) => draw.activate(),
        (true, true) => {
            draw.deactivate();
        }
        _ => {}
    }
}

fn handle_player_collisions(
    player: Single<(Entity, &mut Velocity, &mut Transform), With<Player>>,
    walls: Query<(), With<Wall>>,
    enemies: Query<&EnemyType>,
    collisions: Collisions,
    mut next_state: ResMut<NextState<Playing>>,
) {
    let (player, mut player_v, mut player_t) = player.into_inner();
    for contact_pair in collisions.collisions_with(player) {
        // walls
        if walls.contains(contact_pair.collider1) {
            let normal = contact_pair.manifolds[0].normal;
            let colliion_perp = contact_pair.manifolds[0].normal.perp();
            // zero velocity in direction of impulse
            player_v.0 = player_v.0.dot(colliion_perp) * colliion_perp;
            // make sure player is outside of wall
            if let Some(contact_point) = contact_pair.find_deepest_contact() {
                let mut current_pos = player_t.translation.truncate();
                let z = player_t.translation.z;
                current_pos += normal * contact_point.penetration;
                player_t.translation = current_pos.extend(z);
            }
        }

        if let Ok(typee) = enemies.get(contact_pair.collider1)
            && *typee == EnemyType::White
        {
            next_state.set(Playing::Dying);
        }
    }
}

fn despawn_player(mut commands: Commands, player: Single<Entity, With<Player>>) {
    commands.entity(*player).despawn();
}
