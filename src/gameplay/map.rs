use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use hexx::{Hex, HexLayout, InsetOptions, PlaneMeshBuilder};

use crate::screens::Screen;

pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Gameplay), setup_grid)
            .add_systems(OnExit(Screen::Gameplay), despawn_grid);
    }
}

#[derive(Resource)]
struct Map {
    default_material: Handle<ColorMaterial>,
    grass_material: Handle<ColorMaterial>,
    mesh_handle: Handle<Mesh>,
    cursor_mesh: Handle<Mesh>,
}

#[derive(Component)]
struct MapHex;

/// Hex grid setup
fn setup_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let layout = HexLayout::flat()
        .with_hex_size(50.0)
        .with_origin(vec2(0.0, 0.0));

    let default_material = materials.add(Color::Srgba(bevy::color::palettes::tailwind::GRAY_500));
    let grass_material = materials.add(Color::Srgba(bevy::color::palettes::tailwind::GREEN_800));

    let mesh_handle = meshes.add(hexagonal_plane(&layout));
    let cursor_mesh = meshes.add(border_plane(&layout));

    Hex::ZERO.range(15).for_each(|hex| {
        let pos = layout.hex_to_world_pos(hex);
        let _id = commands
            .spawn((
                MapHex,
                Mesh2d(mesh_handle.clone()),
                MeshMaterial2d(if (hex.x + hex.y) % 2 == 0 {
                    default_material.clone_weak()
                } else {
                    grass_material.clone_weak()
                }),
                Transform::from_xyz(pos.x, pos.y, 0.0),
            ))
            .id();
    });

    commands.insert_resource(Map {
        default_material,
        grass_material,
        mesh_handle,
        cursor_mesh,
    });
}

/// compute mesh from layout
fn hexagonal_plane(layout: &HexLayout) -> Mesh {
    let mesh_info = PlaneMeshBuilder::new(layout)
        .facing(Vec3::Z)
        .center_aligned()
        .build();

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_info.vertices)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_info.normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mesh_info.uvs)
    .with_inserted_indices(Indices::U16(mesh_info.indices))
}

fn border_plane(layout: &HexLayout) -> Mesh {
    let mesh_info = PlaneMeshBuilder::new(layout)
        .facing(Vec3::Z)
        .with_inset_options(InsetOptions {
            keep_inner_face: false,
            scale: 0.2,
            ..default()
        })
        .center_aligned()
        .build();

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_info.vertices)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_info.normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mesh_info.uvs)
    .with_inserted_indices(Indices::U16(mesh_info.indices))
}

fn despawn_grid(mut commands: Commands, hexes: Query<Entity, With<MapHex>>) {
    hexes.iter().for_each(|e| commands.entity(e).despawn());
}
