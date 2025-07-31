use bevy::{prelude::*, window::PrimaryWindow};

use crate::screens::Screen;

pub(crate) struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Gameplay), spawn_player)
            .add_systems(Update, point_player.run_if(in_state(Screen::Gameplay)));
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
    ));
}

fn point_player(
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

    let forward = Vec2::normalize(player_transform.translation.truncate() - target);
    let angle = Vec2::Y.angle_to(forward);
    player_transform.rotation = Quat::from_rotation_z(-angle);

    Ok(())
}
