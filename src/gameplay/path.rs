use bevy::prelude::*;
use geo::LineString;

pub(crate) struct PathPlugin;
impl Plugin for PathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (record_path, find_intersections, draw_path).chain(),
        );
    }
}

#[derive(Component, Default)]
pub(crate) struct DrawPath {
    active: bool,
    path: Option<Entity>,
}

impl DrawPath {
    pub fn active(&self) -> bool {
        self.active
    }

    pub fn path(&self) -> Option<Entity> {
        self.path
    }

    pub fn activate(&mut self) {
        self.active = true;
    }

    pub fn deactivate(&mut self) {
        self.active = false;
        self.path = None;
    }
}

#[derive(Component)]
struct Path {
    // Entity that drew this path
    pen: Entity,
    points: Vec<Vec2>,
}

/// Marker Component for a path that is finished.
#[derive(Component)]
struct ClosedPath;

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
            let new_point = t.translation.truncate();
            let Some(last_point) = path.points.last() else {
                path.points.push(new_point);
                continue;
            };
            if new_point != *last_point {
                path.points.push(new_point);
            }
        } else {
            let points = vec![t.translation.truncate()];
            let path_entity = commands
                .spawn(Path {
                    pen: pencil,
                    points,
                })
                .id();
            draw.path = Some(path_entity);
        }
    }

    Ok(())
}

fn draw_path(paths: Query<&Path>, mut gizmos: Gizmos) {
    for path in &paths {
        for (&point1, &point2) in path.points.iter().zip(path.points.iter().skip(1)) {
            gizmos.line_2d(point1, point2, Color::hsl(360., 1., 1.));
        }
    }
}

/// close the path if needed and stop drawing
fn find_intersections(
    mut commands: Commands,
    mut paths: Query<(Entity, &mut Path), Without<ClosedPath>>,
    mut pens: Query<&mut DrawPath>,
) -> Result<(), BevyError> {
    for (path_entity, mut path) in &mut paths {
        let coords = path
            .points
            .iter()
            .map(|point| (point.x, point.y))
            .collect::<Vec<_>>();
        let line_string = LineString::from(coords);
        let intersections = intersect2d::algorithm::AlgorithmData::<f32>::default()
            .with_stop_at_first_intersection(true)?
            .with_ignore_end_point_intersections(true)?
            .with_lines(line_string.lines())?
            .compute()?;
        for (intersection_point, segment_indicies) in intersections {
            // insert the intersection point and drop points outside the polygon
            let mut new_points = path.points[segment_indicies[0]..segment_indicies[1]].to_vec();
            new_points.push(Vec2::new(intersection_point.x, intersection_point.y));
            path.points = new_points;
            pens.get_mut(path.pen)?.deactivate();
            commands.entity(path_entity).insert(ClosedPath);
        }
    }

    Ok(())
}
