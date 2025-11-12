pub mod height;
pub mod waves;

use avian2d::prelude::*;
use bevy::{prelude::*, sprite_render::Material2dPlugin};
use noiz::prelude::*;

use crate::{
    demo::terrain::height::{TerrainChunk, TerrainMaterial, update_time},
    screens::Screen,
};

pub struct TerrainPlugin;

/// Modeling the terrain as a height map, adding ports, and active entities ontop
impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(OnEnter(Screen::Gameplay), spawn_terrain)
            .add_systems(Update, update_time.run_if(in_state(Screen::Gameplay)))
            .add_plugins(Material2dPlugin::<TerrainMaterial>::default());
    }
}

const CHUNK_SIZE_PIXELS: usize = 4096;

fn spawn_terrain(
    mut commands: Commands,
    mut materials: ResMut<Assets<TerrainMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
) {
    let mesh = meshes.add(Rectangle::new(
        CHUNK_SIZE_PIXELS as f32,
        CHUNK_SIZE_PIXELS as f32,
    ));
    let terrain = generate_chunk();

    let height_tex = images.add(terrain.as_tex());
    let material = materials.add(TerrainMaterial {
        time: Vec4::ZERO,
        height_texture: height_tex,
    });

    commands.spawn((
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_translation(Vec3::ZERO),
    ));

    let land_collider = terrain.land_colliders(Vec2::ZERO);
    for c in land_collider {
        commands.spawn((c.0, c.1, RigidBody::Static));
    }
    warn!("spawn terrain");
}

fn generate_chunk() -> TerrainChunk {
    let mut t = TerrainChunk::zero();
    let mut noise = Noise::<PerCell<OrthoGrid, Random<SNorm, f32>>>::default();
    noise.set_seed(123);
    noise.set_frequency(1.1);

    for y in 0..TerrainChunk::SQUARE {
        for x in 0..TerrainChunk::SQUARE {
            let world_pos = Vec2::new(x as f32, y as f32);
            let height = noise.sample(world_pos);
            // let height = if (y + x * TerrainChunk::SQUARE + 1) % 2 == 0 {
            //     1.0
            // } else {
            //     0.0
            // };

            t.set(x, y, height);
        }
    }

    t
}
