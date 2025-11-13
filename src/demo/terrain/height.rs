use avian2d::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{AsBindGroup, Extent3d},
    sprite_render::Material2d,
};

pub const CHUNK_SIZE_PIXELS: usize = 4096;

pub const SQUARE: usize = 16;
pub const WATER_LEVEL: f32 = 0.5;

#[derive(Component)]
pub struct TerrainChunk {
    heights: Vec<f32>,
}

#[allow(dead_code)]
/// The (0, 0) world coordinate is the center point of the chunk at (0, 0)
pub fn world_2_chunk(p: Vec2) -> (usize, usize) {
    let pixels_per_square = (CHUNK_SIZE_PIXELS / SQUARE) as f32;

    let aligned_chunk = (p / pixels_per_square) + SQUARE as f32 / 2.0;
    let aligned_chunk_index = (aligned_chunk.x as isize, aligned_chunk.y as isize);

    let mut within_chunk_index = (
        aligned_chunk_index.0 % SQUARE as isize,
        aligned_chunk_index.1 % SQUARE as isize,
    );

    // -16, -15, -14, -13, -12, -11, -10, -09, -08, -07, -06, -05, -04, -03, -02, -01, 00, 01, 02, 03
    //  00, -15
    //  00,  01,  02,  03,  04,  05,  06,  07,  08,  09,  10,  11,  12,  13,  14,  15, 00, 01, 02, 03

    if aligned_chunk_index.0 < 0 && within_chunk_index.0 != 0 {
        within_chunk_index.0 += SQUARE as isize;
    }
    if aligned_chunk_index.1 < 0 && within_chunk_index.1 != 0 {
        within_chunk_index.1 += SQUARE as isize;
    }

    (within_chunk_index.0 as usize, within_chunk_index.1 as usize)
}

impl TerrainChunk {
    pub fn zero() -> TerrainChunk {
        let heights = vec![0.0; SQUARE * SQUARE];
        TerrainChunk { heights }
    }

    /// assumes x and y 0..SQUARE
    pub fn set(&mut self, x: usize, y: usize, h: f32) {
        assert!(x < SQUARE);
        assert!(y < SQUARE);

        let i = y * SQUARE + x;
        self.heights[i] = h;
    }

    #[allow(dead_code)]
    pub fn get(&self, x: usize, y: usize) -> f32 {
        assert!(x < SQUARE);
        assert!(y < SQUARE);

        let i = y * SQUARE + x;
        self.heights[i]
    }

    pub fn as_tex(&self) -> Image {
        let height_bytes = self.heights.iter().flat_map(|f| f.to_le_bytes()).collect();
        let mut i = Image::new(
            Extent3d {
                width: SQUARE as u32,
                height: SQUARE as u32,
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
        let mut colliders = Vec::with_capacity(SQUARE * SQUARE);
        let collider_size = (CHUNK_SIZE_PIXELS / SQUARE) as f32;
        for y in 0..SQUARE {
            for x in 0..SQUARE {
                let height = self.get(x, y);
                if height > WATER_LEVEL {
                    let x = x as f32 * collider_size + offset.x
                        - collider_size * (SQUARE / 2) as f32
                        + collider_size * 0.5;
                    let y = y as f32 * collider_size + offset.y
                        - collider_size * (SQUARE / 2) as f32
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
    #[texture(3)]
    #[sampler(4)]
    pub wave_texture: Handle<Image>,
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

mod test {
    #[allow(unused)]
    use bevy::math::Vec2;

    #[allow(unused)]
    use crate::demo::terrain::height::world_2_chunk;

    #[test]
    fn test_coords() {
        assert_eq!(world_2_chunk(Vec2::new(0.0, 0.0)), (8, 8));
        assert_eq!(world_2_chunk(Vec2::new(1.0, 1.0)), (8, 8));
        assert_eq!(world_2_chunk(Vec2::new(1000.0, 1000.0)), (11, 11));
        assert_eq!(world_2_chunk(Vec2::new(2048.0, 2048.0)), (0, 0));
        assert_eq!(world_2_chunk(Vec2::new(-2048.0, -2048.0)), (0, 0));
        assert_eq!(world_2_chunk(Vec2::new(-1024.0, -1024.0)), (4, 4));
        assert_eq!(world_2_chunk(Vec2::new(-64.0, -64.0)), (7, 7));
        assert_eq!(world_2_chunk(Vec2::new(-256.0, -256.0) * 7.0), (1, 1));
        assert_eq!(world_2_chunk(Vec2::new(-256.0, -256.0) * 1.0), (7, 7));
        assert_eq!(world_2_chunk(Vec2::new(-256.0, -256.0) * 2.0), (6, 6));
        assert_eq!(world_2_chunk(Vec2::new(-256.0, -256.0) * 8.0), (0, 0));
        assert_eq!(world_2_chunk(Vec2::new(-256.0, -256.0) * 9.0), (15, 15));
    }
}
