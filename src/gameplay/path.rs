use avian2d::prelude::{Collider, LinearVelocity, RigidBody};
use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};
use geo::{LineString, Point, Polygon, prelude::Contains};

use crate::gameplay::enemy::{Enemy, EnemyHandles, EnemyType, SpawnEnemies};

pub(crate) struct PathPlugin;
impl Plugin for PathPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LivePaths>().add_systems(
            FixedUpdate,
            (
                record_path,
                find_intersections,
                check_areas,
                draw_path,
                despawwn_old_paths,
            )
                .chain(),
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

    pub fn is_active_path(&self, path: Entity) -> bool {
        self.path.is_some_and(|e| e == path)
    }
}

#[derive(Component)]
#[component(on_insert = add_to_active_paths)]
#[component(on_remove = remove_from_active_paths)]
struct Path {
    // Entity that drew this path
    #[allow(dead_code)]
    pen: Entity,
    points: Vec<Vec2>,
    remainder: Option<Vec<Vec2>>,
}

fn add_to_active_paths(mut world: DeferredWorld, context: HookContext) {
    world.resource_mut::<LivePaths>().0.push(context.entity);
}

fn remove_from_active_paths(mut world: DeferredWorld, context: HookContext) {
    let mut active_paths = world.resource_mut::<LivePaths>();
    active_paths.0.retain(|e| *e != context.entity);
}

impl Path {
    fn to_line_string(&self) -> LineString<f32> {
        let coords = self
            .points
            .iter()
            .map(|point| (point.x, point.y))
            .collect::<Vec<_>>();
        LineString::from(coords)
    }

    fn swap_remainder(&mut self) -> bool {
        let Some(remainder) = self.remainder.take() else {
            return false;
        };

        self.points = remainder;
        true
    }
}

/// List of paths that currently exist. Used to remove the oldest path first
#[derive(Resource, Default)]
struct LivePaths(Vec<Entity>);

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
                commands.entity(path_entity).with_children(|child| {
                    child.spawn((
                        Collider::segment(new_point, *last_point),
                        RigidBody::Kinematic,
                    ));
                });
                path.points.push(new_point);
            }
        } else {
            let points = vec![t.translation.truncate()];
            let path_entity = commands
                .spawn((
                    Path {
                        pen: pencil,
                        points,
                        remainder: None,
                    },
                    Transform::default(),
                ))
                .id();
            draw.path = Some(path_entity);
        }
    }

    Ok(())
}

fn draw_path(
    paths: Query<(Entity, &Path)>,
    mut gizmos: Gizmos,
    player: Single<&DrawPath>,
    path_list: Res<LivePaths>,
) {
    for (path_entity, path) in &paths {
        for (&point1, &point2) in path.points.iter().zip(path.points.iter().skip(1)) {
            let color = if let Some(active_path) = player.path()
                && path_entity == active_path
            {
                Color::hsl(360., 1., 1.)
            } else {
                let index = path_list
                    .0
                    .iter()
                    .position(|e| *e == path_entity)
                    .unwrap_or_default();
                let shade = path_list.0.len() - index;
                Color::hsl(360., 0., 0.6 - 0.075 * shade as f32)
            };
            gizmos.line_2d(point1, point2, color);
        }
    }
}

/// close the path if needed and stop drawing
fn find_intersections(
    mut commands: Commands,
    mut paths: Query<(Entity, &mut Path), Without<ClosedPath>>,
) -> Result<(), BevyError> {
    for (path_entity, mut path) in &mut paths {
        let mut intersections = intersect2d::algorithm::AlgorithmData::<f32>::default()
            .with_stop_at_first_intersection(true)?
            .with_ignore_end_point_intersections(true)?
            .with_lines(path.to_line_string().lines())?
            .compute()?;
        if let Some((intersection_point, segment_indicies)) = intersections.next() {
            // insert the intersection point and drop points outside the polygon
            let mut new_points = path.points[segment_indicies[0]..segment_indicies[1]].to_vec();
            new_points.push(Vec2::new(intersection_point.x, intersection_point.y));

            // save the remainder
            let mut remainder = path.points[0..segment_indicies[0]].to_vec();
            remainder.push(Vec2::new(intersection_point.x, intersection_point.y));
            remainder.extend(&path.points[segment_indicies[1]..]);
            if remainder.len() > new_points.len() {
                path.remainder = Some(remainder);
            }

            path.points = new_points;
            commands.entity(path_entity).insert(ClosedPath);
        }
    }

    Ok(())
}

fn check_areas(
    mut commands: Commands,
    mut paths: Query<(Entity, &mut Path), (With<ClosedPath>, Changed<Path>)>,
    enemies: Query<(Entity, &Transform, &EnemyType, &LinearVelocity), With<Enemy>>,
    handles: Res<EnemyHandles>,
    mut spawn_enemies: EventWriter<SpawnEnemies>,
    mut pen: Single<&mut DrawPath>,
) {
    for (e, mut path) in &mut paths {
        let polygon = Polygon::new(path.to_line_string(), vec![]);
        let mut surrounded = Vec::new();
        for (enemy_entity, transform, enemy_type, velocity) in &enemies {
            if polygon.contains(&Point::new(
                transform.translation.x,
                transform.translation.y,
            )) {
                surrounded.push((enemy_entity, *enemy_type, *transform, velocity));
            }
        }

        match surrounded.len() {
            0 => {
                if pen.is_active_path(e) {
                    if !path.swap_remainder() {
                        pen.deactivate();
                    } else {
                        commands.entity(e).remove::<ClosedPath>();
                    }
                }
            }
            1 => {
                pen.deactivate();
            }
            2 | 3 => {
                if let Some((typ, new_t, new_v)) = EnemyType::check_combine(surrounded.iter()) {
                    typ.spawn(&mut commands, new_t, new_v, &handles);
                    for (enemy, ..) in surrounded {
                        commands.entity(enemy).despawn();
                    }
                    if typ == EnemyType::White {
                        spawn_enemies.write(SpawnEnemies);
                    }
                } else {
                    // explode the items
                    // commands.entity(e).despawn();
                }
                pen.deactivate();
            }
            _ => {
                pen.deactivate();
                // explode the items
                // commands.entity(e).despawn();
            }
        }
    }
}

fn despawwn_old_paths(mut commands: Commands, live_paths: Res<LivePaths>) {
    if live_paths.0.len() > 4 {
        commands.entity(live_paths.0[0]).despawn();
    }
}
