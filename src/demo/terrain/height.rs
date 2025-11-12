use avian2d::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{AsBindGroup, Extent3d},
    sprite_render::Material2d,
};

use crate::demo::terrain::CHUNK_SIZE_PIXELS;

pub struct TerrainChunk {
    heights: Vec<f32>,
}

impl TerrainChunk {
    pub const SQUARE: usize = 16;
    pub const WATER_LEVEL: f32 = 0.5;

    pub fn zero() -> TerrainChunk {
        let heights = vec![0.0; TerrainChunk::SQUARE * TerrainChunk::SQUARE];
        TerrainChunk { heights }
    }

    /// assumes x and y 0..SQUARE
    pub fn set(&mut self, x: usize, y: usize, h: f32) {
        assert!(x < TerrainChunk::SQUARE);
        assert!(y < TerrainChunk::SQUARE);

        let i = y * TerrainChunk::SQUARE + x;
        self.heights[i] = h;
    }

    #[allow(dead_code)]
    pub fn get(&self, x: usize, y: usize) -> f32 {
        assert!(x < TerrainChunk::SQUARE);
        assert!(y < TerrainChunk::SQUARE);

        let i = y * TerrainChunk::SQUARE + x;
        self.heights[i]
    }

    pub fn as_tex(&self) -> Image {
        let height_bytes = self.heights.iter().flat_map(|f| f.to_le_bytes()).collect();
        let mut i = Image::new(
            Extent3d {
                width: TerrainChunk::SQUARE as u32,
                height: TerrainChunk::SQUARE as u32,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            height_bytes,
            bevy::render::render_resource::TextureFormat::R32Float,
            RenderAssetUsages::all(),
        );
        i.sampler = ImageSampler::nearest();

        i
    }

    pub fn land_colliders(&self, offset: Vec2) -> Vec<(Collider, Transform)> {
        let mut colliders = Vec::with_capacity(TerrainChunk::SQUARE * TerrainChunk::SQUARE);
        let collider_size = (CHUNK_SIZE_PIXELS / TerrainChunk::SQUARE) as f32;
        for y in 0..TerrainChunk::SQUARE {
            for x in 0..TerrainChunk::SQUARE {
                let height = self.get(x, y);
                if height > TerrainChunk::WATER_LEVEL {
                    let x = x as f32 * collider_size + offset.x
                        - collider_size * (TerrainChunk::SQUARE / 2) as f32
                        + collider_size * 0.5;
                    let y = y as f32 * collider_size + offset.y
                        - collider_size * (TerrainChunk::SQUARE / 2) as f32
                        + collider_size * 0.5;
                    colliders.push((
                        Collider::rectangle(collider_size, collider_size),
                        Transform::from_xyz(x, y, 0.0),
                    ));
                }
            }
        }

        colliders.shrink_to_fit();
        colliders
    }
}

pub fn update_time(
    time: Res<Time>,
    mut terrain_materials: ResMut<Assets<TerrainMaterial>>,
    terrain_chunks: Query<&MeshMaterial2d<TerrainMaterial>>,
) {
    for c in terrain_chunks.iter() {
        if let Some(m) = terrain_materials.get_mut(c.0.id()) {
            m.time = Vec4::new(time.elapsed_secs(), 0.0, 0.0, 0.0);
        }
    }
}

/// Contains the render information for a single chunk
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct TerrainMaterial {
    #[uniform(0)]
    pub time: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub height_texture: Handle<Image>,
}

const FRAGMENT_SHADER_ASSET_PATH: &str = "shaders/terrain.wesl";

impl Material2d for TerrainMaterial {
    fn vertex_shader() -> bevy::shader::ShaderRef {
        bevy::shader::ShaderRef::Default
    }

    fn fragment_shader() -> bevy::shader::ShaderRef {
        FRAGMENT_SHADER_ASSET_PATH.into()
    }

    fn depth_bias(&self) -> f32 {
        0.0
    }

    fn alpha_mode(&self) -> bevy::sprite_render::AlphaMode2d {
        bevy::sprite_render::AlphaMode2d::Opaque
    }
}
