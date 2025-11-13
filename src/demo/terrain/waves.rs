use bevy::{
    asset::RenderAssetUsages, image::ImageSampler, prelude::*, render::render_resource::Extent3d,
};

use crate::demo::terrain::height::{SQUARE, TerrainChunk, WATER_LEVEL, world_2_chunk};

#[derive(Component)]
pub struct Waves {
    directions: Vec<Vec2>,
}

impl Waves {
    #[allow(dead_code)]
    pub fn init(terrain: &TerrainChunk) -> Waves {
        let directions = vec![Vec2::ZERO; SQUARE * SQUARE];
        let mut w = Waves { directions };
        // in increasing squares up to x size check the surrounding
        for x in 0..SQUARE {
            for y in 0..SQUARE {
                if terrain.get(x, y) > WATER_LEVEL {
                    continue;
                }
                let current_vec = Vec2::new(x as f32, y as f32);
                for r in 1..5 {
                    let mut found: Vec<Vec2> = Vec::with_capacity(10);
                    for square_index in square_neighborhood(x as isize, y as isize, r) {
                        let height = terrain.get(square_index.0, square_index.1);
                        if height > WATER_LEVEL {
                            found.push(
                                current_vec
                                    - Vec2::new(square_index.0 as f32, square_index.1 as f32),
                            );
                        }
                    }
                    if !found.is_empty() {
                        w.set(x, y, avg(&found));
                        break;
                    }
                }
            }
        }

        w
    }

    pub fn as_tex(&self) -> Image {
        let bytes = self
            .directions
            .iter()
            .flat_map(|d| {
                let x = (d.x as f32).to_le_bytes();
                let y = (d.y as f32).to_le_bytes();
                let mut r = Vec::with_capacity(x.len() + y.len());
                r.extend_from_slice(&x);
                r.extend_from_slice(&y);
                r
            })
            .collect();

        let mut i = Image::new(
            Extent3d {
                width: SQUARE as u32,
                height: SQUARE as u32,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            bytes,
            bevy::render::render_resource::TextureFormat::Rg32Float,
            RenderAssetUsages::all(),
        );

        i.sampler = ImageSampler::nearest();

        i
    }

    /// returning height, and rising(true) or lowering(false)
    /// x, y in world space
    #[allow(dead_code)]
    pub fn wave_height(&self, p: Vec2, t: f32) -> (Vec2, f32, bool) {
        let index = world_2_chunk(p);
        let dir = self.get(index.0, index.1);
        let dir = Vec2::new(dir.x as f32, dir.y as f32).normalize();

        let v = (p * 0.05).dot(dir);

        let t1 = (v + (t * 5.5)).sin();
        let t2 = (v + (t * 5.5) + 0.001).sin();

        (dir, t1, t1 < t2)
    }

    #[allow(dead_code)]
    pub fn format(&self) -> String {
        let mut s = String::new();
        for x in 0..SQUARE {
            for y in 0..SQUARE {
                let v = self.get(x, y);
                let v_x = v.x;
                let v_y = v.y;
                s.push_str(format!("|{v_x:02.2},{v_y:02.2}|").as_str());
            }
            s.push('\n');
        }
        s
    }

    /// assumes x and y 0..SQUARE
    #[allow(dead_code)]
    pub fn set(&mut self, x: usize, y: usize, dir: Vec2) {
        assert!(x < SQUARE);
        assert!(y < SQUARE);

        let i = y * SQUARE + x;
        self.directions[i] = dir;
    }

    #[allow(dead_code)]
    pub fn get(&self, x: usize, y: usize) -> Vec2 {
        assert!(x < SQUARE);
        assert!(y < SQUARE);

        let i = y * SQUARE + x;
        self.directions[i]
    }
}

fn square_neighborhood(cx: isize, cy: isize, r: isize) -> impl Iterator<Item = (usize, usize)> {
    let x0 = (cx - r).max(0);
    let y0 = (cy - r).max(0);
    let x1 = (cx + r).min(SQUARE as isize - 1);
    let y1 = (cy + r).min(SQUARE as isize - 1);

    (y0..=y1).flat_map(move |y| (x0..=x1).map(move |x| (x as usize, y as usize)))
}

fn avg(vecs: &[Vec2]) -> Vec2 {
    vecs.iter().sum::<Vec2>() / (vecs.len() as f32)
}

mod test {
    #[allow(unused)]
    use bevy::math::Vec2;

    #[allow(unused)]
    use crate::demo::terrain::height::SQUARE;
    #[allow(unused)]
    use crate::demo::terrain::{height::TerrainChunk, waves::Waves};

    #[test]
    fn waves_init() {
        let mut terrain = TerrainChunk::zero();
        for x in [0, SQUARE - 1] {
            for y in 0..SQUARE {
                terrain.set(x, y, 1.0);
            }
        }
        for y in [0, SQUARE - 1] {
            for x in 0..SQUARE {
                terrain.set(x, y, 1.0);
            }
        }

        let waves = Waves::init(&terrain);
        println!("{}", waves.format());

        assert_eq!(waves.get(0, 0), Vec2::ZERO);
        assert_eq!(waves.get(1, 0), Vec2::ZERO);
        assert_eq!(waves.get(0, 1), Vec2::ZERO);
        assert_eq!(waves.get(1, 1), Vec2::new(-1.0, -1.0)); // avg of |(1,1), (0,1)|,|(1,1), (1,0)|,
    }
}
