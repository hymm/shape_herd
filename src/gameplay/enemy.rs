use std::f32::consts::PI;

use avian2d::prelude::{
    AngularVelocity, CoefficientCombine, Collider, Friction, LinearVelocity, Restitution, RigidBody,
};
use bevy::color::palettes::tailwind;
use bevy::math::ops::cos;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::Rng;

use crate::gameplay::rng_bag::RngBag;
use crate::screens::Screen;

pub(crate) struct EnemyPlugin;
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnemyHandles>()
            .add_event::<SpawnEnemies>()
            .add_systems(
                OnEnter(Screen::Gameplay),
                |mut spawn: EventWriter<SpawnEnemies>| {
                    spawn.write(SpawnEnemies);
                },
            )
            .add_systems(Update, spawn_enemies.run_if(in_state(Screen::Gameplay)));
    }
}

/// Marker component for an enemy
#[derive(Component)]
pub struct Enemy;

#[derive(Event)]
pub struct SpawnEnemies;

#[derive(Resource)]
pub(crate) struct EnemyHandles {
    red_mesh: Handle<Mesh>,
    red_material: Handle<ColorMaterial>,
    green_mesh: Handle<Mesh>,
    green_material: Handle<ColorMaterial>,
    blue_mesh: Handle<Mesh>,
    blue_material: Handle<ColorMaterial>,
    purple_mesh: Handle<Mesh>,
    purple_material: Handle<ColorMaterial>,
    yellow_mesh: Handle<Mesh>,
    yellow_material: Handle<ColorMaterial>,
    cyan_mesh: Handle<Mesh>,
    cyan_material: Handle<ColorMaterial>,
    white_mesh: Handle<Mesh>,
    white_material: Handle<ColorMaterial>,
}

impl FromWorld for EnemyHandles {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let red_mesh = meshes.add(EnemyType::Red.mesh());
        let blue_mesh = meshes.add(EnemyType::Blue.mesh());
        let green_mesh = meshes.add(EnemyType::Green.mesh());
        let purple_mesh = meshes.add(EnemyType::Purple.mesh());
        let yellow_mesh = meshes.add(EnemyType::Yellow.mesh());
        let cyan_mesh = meshes.add(EnemyType::Cyan.mesh());
        let white_mesh = meshes.add(EnemyType::White.mesh());

        let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
        let red_material = materials.add(EnemyType::Red.material());
        let blue_material = materials.add(EnemyType::Blue.material());
        let green_material = materials.add(EnemyType::Green.material());
        let purple_material = materials.add(EnemyType::Purple.material());
        let yellow_material = materials.add(EnemyType::Yellow.material());
        let cyan_material = materials.add(EnemyType::Cyan.material());
        let white_material = materials.add(EnemyType::White.material());

        Self {
            red_mesh,
            red_material,
            green_mesh,
            green_material,
            blue_mesh,
            blue_material,
            purple_mesh,
            purple_material,
            yellow_mesh,
            yellow_material,
            cyan_mesh,
            cyan_material,
            white_mesh,
            white_material,
        }
    }
}

impl EnemyHandles {
    fn mesh(&self, t: EnemyType) -> Handle<Mesh> {
        match t {
            EnemyType::Red => self.red_mesh.clone(),
            EnemyType::Green => self.green_mesh.clone(),
            EnemyType::Blue => self.blue_mesh.clone(),
            EnemyType::Purple => self.purple_mesh.clone(),
            EnemyType::Yellow => self.yellow_mesh.clone(),
            EnemyType::Cyan => self.cyan_mesh.clone(),
            EnemyType::White => self.white_mesh.clone(),
        }
    }

    fn material(&self, t: EnemyType) -> Handle<ColorMaterial> {
        match t {
            EnemyType::Red => self.red_material.clone(),
            EnemyType::Green => self.green_material.clone(),
            EnemyType::Blue => self.blue_material.clone(),
            EnemyType::Purple => self.purple_material.clone(),
            EnemyType::Yellow => self.yellow_material.clone(),
            EnemyType::Cyan => self.cyan_material.clone(),
            EnemyType::White => self.white_material.clone(),
        }
    }
}

#[derive(Component, Clone, Copy, PartialEq)]
pub enum EnemyType {
    Red,
    Green,
    Blue,
    Purple,
    Yellow,
    Cyan,
    White,
}

impl EnemyType {
    pub fn spawn(
        self,
        commands: &mut Commands,
        transform: Transform,
        velocity: LinearVelocity,
        handles: &EnemyHandles,
    ) {
        let mesh = handles.mesh(self);
        let material = handles.material(self);
        commands.spawn((
            self,
            Enemy,
            transform,
            MeshMaterial2d(material),
            Mesh2d(mesh),
            RigidBody::Dynamic,
            velocity,
            AngularVelocity::default(),
            Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
            Collider::circle(10.),
            Restitution::new(0.8),
        ));
    }

    fn material(&self) -> ColorMaterial {
        let color = match self {
            EnemyType::Red => tailwind::RED_500,
            EnemyType::Green => tailwind::GREEN_500,
            EnemyType::Blue => tailwind::BLUE_500,
            EnemyType::Purple => tailwind::PURPLE_500,
            EnemyType::Yellow => tailwind::YELLOW_500,
            EnemyType::Cyan => tailwind::CYAN_500,
            EnemyType::White => tailwind::GRAY_100,
        };
        ColorMaterial {
            color: color.into(),
            ..default()
        }
    }

    fn mesh(&self) -> Mesh {
        const SHAPE_LENGTH: f32 = 20.;
        match self {
            EnemyType::Red => {
                let triangle_height = SHAPE_LENGTH * cos(PI / 3.);
                Triangle2d::new(
                    Vec2::new(0., triangle_height),
                    Vec2::new(-SHAPE_LENGTH / 2., -triangle_height),
                    Vec2::new(SHAPE_LENGTH / 2., -triangle_height),
                )
                .into()
            }
            EnemyType::Green => RegularPolygon::new(0.55 * SHAPE_LENGTH, 6).into(),
            EnemyType::Blue => Rectangle::new(SHAPE_LENGTH, SHAPE_LENGTH).into(),
            EnemyType::Purple => RegularPolygon::new(1.1 * SHAPE_LENGTH, 6).into(),
            EnemyType::Yellow => Rectangle::new(2. * SHAPE_LENGTH, 2. * SHAPE_LENGTH).into(),
            EnemyType::Cyan => {
                let triangle_height = 2. * SHAPE_LENGTH * cos(PI / 3.);
                Triangle2d::new(
                    Vec2::new(0., triangle_height),
                    Vec2::new(-SHAPE_LENGTH, -triangle_height),
                    Vec2::new(SHAPE_LENGTH, -triangle_height),
                )
                .into()
            }
            EnemyType::White => Circle::new(0.75 * SHAPE_LENGTH).into(),
        }
    }

    fn pair_combine(&self, other: Self) -> Option<Self> {
        use EnemyType::*;
        match (self, other) {
            // combine primaries
            (Red, Green) | (Green, Red) => Some(Yellow),
            (Red, Blue) | (Blue, Red) => Some(Purple),
            (Green, Blue) | (Blue, Green) => Some(Cyan),
            // complements
            (Red, Cyan) | (Cyan, Red) => Some(White),
            (Green, Purple) | (Purple, Green) => Some(White),
            (Blue, Yellow) | (Yellow, Blue) => Some(White),
            _ => None,
        }
    }

    fn is_primary(&self) -> bool {
        use EnemyType::*;
        matches!(self, Red | Green | Blue)
    }

    /// returns None if they can't combine
    pub fn check_combine<'a>(
        mut enemies: impl ExactSizeIterator<
            Item = &'a (Entity, EnemyType, Transform, &'a LinearVelocity),
        >,
    ) -> Option<(EnemyType, Transform, LinearVelocity)> {
        match enemies.len() {
            0 | 1 => None,
            2 => {
                let Some((_, enemy_type, t, v)) = enemies.next() else {
                    unreachable!();
                };

                let Some((_, next_type, next_t, next_v)) = enemies.next() else {
                    unreachable!();
                };

                let new_type = enemy_type.pair_combine(*next_type)?;

                let new_t = Transform::from_translation((t.translation + next_t.translation) / 2.);
                let new_v = LinearVelocity((v.0 + next_v.0) / 2.);

                Some((new_type, new_t, new_v))
            }
            3 => {
                let mut has_red = false;
                let mut has_green = false;
                let mut has_blue = false;
                let mut is_primary = None;
                let mut new_t = Vec2::default();
                let mut new_v = Vec2::default();
                for (_, typ, t, v) in enemies {
                    if is_primary.is_none() {
                        is_primary = Some(typ.is_primary());
                    }

                    match typ {
                        EnemyType::Green => has_green = true,
                        EnemyType::Blue => has_blue = true,
                        EnemyType::Red => has_red = true,
                        _ => {}
                    }

                    new_t += t.translation.truncate();
                    new_v += v.0;
                }

                if has_red && has_green && has_blue {
                    Some((
                        EnemyType::White,
                        Transform::from_translation((new_t / 3.).extend(0.0)),
                        LinearVelocity(new_v / 3.),
                    ))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

fn spawn_enemies(
    mut commands: Commands,
    handles: Res<EnemyHandles>,
    mut spawn: EventReader<SpawnEnemies>,
    enemies: Query<(), With<Enemy>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let mut rng_bag = RngBag::new(vec![EnemyType::Red, EnemyType::Blue, EnemyType::Green]);
    let mut rng = rand::thread_rng();
    for _ in spawn.read() {
        let item_count = if enemies.iter().len() < 3 { 3 } else { 6 };
        for _ in 0..item_count {
            let typ = rng_bag.get();
            const MAX_VELOCITY: f32 = 100.0;
            let max_t = window.size() / 2. - 20.;
            typ.spawn(
                &mut commands,
                Transform::from_xyz(
                    rng.gen_range(-max_t.x..max_t.x),
                    rng.gen_range(-max_t.y..max_t.y),
                    0.,
                ),
                LinearVelocity(Vec2::new(
                    rng.gen_range(-MAX_VELOCITY..MAX_VELOCITY),
                    rng.gen_range(-MAX_VELOCITY..MAX_VELOCITY),
                )),
                &handles,
            );
        }
    }
}
