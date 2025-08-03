use std::f32::consts::PI;

use avian2d::prelude::{Collider, ColliderDisabled, LinearVelocity, RigidBody};
use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    math::ops::exp,
    platform::collections::HashMap,
    prelude::*,
};
use geo::{LineString, Point, Polygon, prelude::Contains};
use rand::Rng;

use crate::{
    gameplay::{
        DespawnSet,
        enemy::{Enemy, EnemyHandles, EnemyType, SpawnEnemies},
    },
    screens::Screen,
};

pub(crate) struct PathPlugin;
impl Plugin for PathPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LivePaths>()
            .add_systems(
                FixedUpdate,
                (
                    record_path,
                    find_intersections,
                    check_areas,
                    draw_path,
                    despawwn_old_paths,
                    animate_combining,
                )
                    .chain(),
            )
            .add_systems(
                OnExit(Screen::Gameplay),
                despawn_all_paths.in_set(DespawnSet),
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
    enemies: Query<
        (Entity, &Transform, &EnemyType, &LinearVelocity),
        (With<Enemy>, Without<ColliderDisabled>),
    >,
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
            _ => {
                let mut combines = Vec::new();
                let mut explode = Vec::new();

                while let Some((check_entity, typ, _t, v)) = surrounded.pop() {
                    let mut entities = vec![check_entity];

                    // check for complements
                    let complement_pos = surrounded
                        .iter()
                        .position(|(_, other_type, ..)| typ.complement() == *other_type);
                    if let Some(pos) = complement_pos {
                        let (comp_e, comp_typ, _comp_t, comp_v) = surrounded.remove(pos);
                        entities.push(comp_e);

                        combines.push(Combine {
                            entities,
                            new_type: typ.pair_combine(comp_typ).unwrap(),
                            velocity: (**v + **comp_v) / 2.,
                        });
                        continue;
                    }

                    // check for full set of primaries
                    let mut required = vec![EnemyType::Red, EnemyType::Blue, EnemyType::Green];
                    if !required.contains(&typ) {
                        explode.push(check_entity);
                        continue;
                    }
                    required.retain(|t| *t != typ);
                    let Some(pos) = surrounded
                        .iter()
                        .position(|(_, typee, ..)| *typee == required[0] || *typee == required[1])
                    else {
                        explode.push(check_entity);
                        continue;
                    };
                    let result1 = surrounded.remove(pos);
                    required.retain(|t| *t != result1.1);

                    let Some(pos) = surrounded
                        .iter()
                        .position(|(_, typee, ..)| *typee == required[0])
                    else {
                        entities.push(result1.0);
                        let new_type = typ.pair_combine(result1.1).unwrap();
                        combines.push(Combine {
                            entities,
                            new_type,
                            velocity: (**v + **result1.3) / 2.0,
                        });
                        continue;
                    };
                    let result2 = surrounded.remove(pos);
                    entities.push(result1.0);
                    entities.push(result2.0);

                    combines.push(Combine {
                        entities,
                        new_type: EnemyType::White,
                        velocity: (**v + **result1.3 + **result2.3) / 3.0,
                    });
                }

                commands.spawn(AnimateCombining::Initialize { combines, explode });
            }
        }
    }
}
/// Combine these entites into one enemy
struct Combine {
    entities: Vec<Entity>,
    new_type: EnemyType,
    velocity: Vec2,
}

#[derive(Component)]
enum AnimateCombining {
    Initialize {
        combines: Vec<Combine>,
        explode: Vec<Entity>,
    },
    MoveToCenter {
        center: Vec2,
        target_positions: HashMap<Entity, Vec2>,
        combines: Vec<Combine>,
        explode: Vec<Entity>,
    },
    Eject {
        center: Vec2,
        combines: Vec<Combine>,
        explode: Vec<Entity>,
    },
    Done {
        center: Vec2,
        combines: Vec<Combine>,
    },
    None,
}

impl AnimateCombining {
    fn transition_to_move_to_center(
        &mut self,
        center: Vec2,
        target_positions: HashMap<Entity, Vec2>,
    ) {
        let old = std::mem::replace(self, Self::None);

        match old {
            AnimateCombining::Initialize { combines, explode } => {
                *self = AnimateCombining::MoveToCenter {
                    center,
                    target_positions,
                    combines,
                    explode,
                }
            }
            _ => unreachable!(),
        }
    }

    fn transition_to_eject(&mut self) {
        let old = std::mem::replace(self, Self::None);

        match old {
            AnimateCombining::MoveToCenter {
                combines,
                explode,
                center,
                ..
            } => {
                *self = AnimateCombining::Eject {
                    center,
                    combines,
                    explode,
                }
            }
            _ => unreachable!(),
        }
    }

    fn transition_to_done(&mut self) {
        let old = std::mem::replace(self, Self::None);

        match old {
            AnimateCombining::Eject {
                combines, center, ..
            } => *self = AnimateCombining::Done { center, combines },
            _ => unreachable!(),
        }
    }
}

#[derive(Deref, DerefMut)]
struct TimeoutTimer(Timer);
impl Default for TimeoutTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.5, TimerMode::Once))
    }
}

fn animate_combining(
    mut commands: Commands,
    time: Res<Time>,
    mut animations: Query<(Entity, &mut AnimateCombining)>,
    mut enemies: Query<(&mut Transform, &mut LinearVelocity), With<Enemy>>,
    mut spawn_enemies: EventWriter<SpawnEnemies>,
    paths: Query<Entity, With<Path>>,
    handles: Res<EnemyHandles>,
    mut animation_timeout: Local<TimeoutTimer>,
) {
    const RADIUS: f32 = 10.0;
    let mut rng = rand::thread_rng();
    for (animate_entity, mut anim) in &mut animations {
        match *anim {
            AnimateCombining::Initialize {
                ref combines,
                ref explode,
            } => {
                let mut center_sum = Vec2::default();
                let entity_count = combines
                    .iter()
                    .flat_map(|combine| combine.entities.iter().copied())
                    .chain(explode.iter().copied())
                    .count();
                for e in combines
                    .iter()
                    .flat_map(|combine| combine.entities.iter().copied())
                    .chain(explode.iter().copied())
                {
                    // disable collision boxes
                    commands.entity(e).insert(ColliderDisabled);

                    // calculate target position
                    center_sum += enemies
                        .get(e)
                        .map(|(t, ..)| t)
                        .copied()
                        .unwrap_or_default()
                        .translation
                        .truncate();
                }

                let center = center_sum / entity_count as f32;

                let target_positions = combines
                    .iter()
                    .flat_map(|combine| combine.entities.iter().copied())
                    .chain(explode.iter().copied())
                    .enumerate()
                    .map(|(i, e)| {
                        let offset_vec = Vec2::from_angle(2. * PI * i as f32 / entity_count as f32);
                        let target = center + RADIUS * offset_vec;
                        (e, target)
                    })
                    .collect();

                anim.transition_to_move_to_center(center, target_positions);
            }
            AnimateCombining::MoveToCenter {
                ref target_positions,
                ..
            } => {
                for (entity, target) in target_positions.iter() {
                    let mut transform = enemies.get_mut(*entity).map(|(t, ..)| t).unwrap();
                    transform.translation = transform
                        .translation
                        .lerp(target.extend(0.0), 1. - exp(-10. * time.delta_secs()));
                }

                if animation_timeout.tick(time.delta()).finished() {
                    animation_timeout.reset();
                    anim.transition_to_eject();
                }
            }
            AnimateCombining::Eject { ref explode, .. } => {
                // spit out exploded bits
                for e in explode {
                    commands.entity(*e).remove::<ColliderDisabled>();
                    let Ok((_, mut v)) = enemies.get_mut(*e) else {
                        continue;
                    };

                    **v = Vec2::new(
                        rng.gen_range(-1000.0..1000.0),
                        rng.gen_range(-1000.0..1000.0),
                    );
                }

                if !explode.is_empty() {
                    for e in &paths {
                        commands.entity(e).despawn();
                    }
                }

                anim.transition_to_done();
            }
            AnimateCombining::Done {
                ref center,
                ref combines,
            } => {
                for combine in combines {
                    for e in &combine.entities {
                        commands.entity(*e).despawn();
                    }

                    combine.new_type.spawn(
                        &mut commands,
                        Transform::from_translation(center.extend(0.0)),
                        LinearVelocity(combine.velocity),
                        &handles,
                    );

                    if combine.new_type == EnemyType::White {
                        spawn_enemies.write(SpawnEnemies);
                    }
                }

                commands.entity(animate_entity).despawn();
            }
            AnimateCombining::None => unreachable!(),
        }
    }
}

fn despawwn_old_paths(mut commands: Commands, live_paths: Res<LivePaths>) {
    if live_paths.0.len() > 4 {
        commands.entity(live_paths.0[0]).despawn();
    }
}

fn despawn_all_paths(mut commands: Commands, paths: Query<Entity, With<Path>>) {
    for e in &paths {
        commands.entity(e).despawn();
    }
}
