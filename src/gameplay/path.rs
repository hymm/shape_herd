use bevy::prelude::*;

pub(crate) struct PathPlugin;
impl Plugin for PathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (record_path, draw_path).chain());
    }
}

#[derive(Component)]
pub(crate) struct DrawPath {
    active: bool,
    path: Option<Entity>,
}

impl Default for DrawPath {
    fn default() -> Self {
        Self {
            active: false,
            path: None,
        }
    }
}

impl DrawPath {
    pub fn active(&self) -> bool {
        self.active
    }

    pub fn activate(&mut self) {
        self.active = true;
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }
}

#[derive(Component)]
struct Path {
    // Entity that draws this path
    pencil: Entity,
    points: Vec<Vec2>,
}

fn record_path(
    mut commands: Commands,
    mut pencils: Query<(Entity, &mut DrawPath, &Transform)>,
    mut paths: Query<&mut Path>,
) -> Result<(), BevyError> {
    for (pencil, mut draw, t) in &mut pencils {
        if !draw.active {
            continue;
        }

        if let Some(path_entity) = draw.path
            && let Ok(mut path) = paths.get_mut(path_entity)
        {
            path.points.push(t.translation.truncate());
        } else {
            let points = vec![t.translation.truncate()];
            let path_entity = commands.spawn(Path { pencil, points }).id();
            draw.path = Some(path_entity);
        }
    }

    Ok(())
}

fn draw_path(paths: Query<&mut Path>, mut gizmos: Gizmos) {
    for path in &paths {
        for (&point1, &point2) in path.points.iter().zip(path.points.iter().skip(1)) {
            gizmos.line_2d(point1, point2, Color::hsl(360., 1., 1.));
        }
    }
}
