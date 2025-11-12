use bevy::prelude::*;

use crate::demo::terrain::height::TerrainChunk;

pub struct Waves {
    directions: Vec<IVec2>,
}

impl Waves {
    pub fn init(terrain: &TerrainChunk) -> Waves {
        let directions = vec![IVec2::ZERO; TerrainChunk::SQUARE * TerrainChunk::SQUARE];
        let mut w = Waves { directions };
        for x in 0..TerrainChunk::SQUARE as isize {
            for y in 0..TerrainChunk::SQUARE as isize {
                let mut min_x_d = TerrainChunk::SQUARE as isize;
                let mut x_d = if x > TerrainChunk::SQUARE as isize / 2 {
                    -x
                } else {
                    TerrainChunk::SQUARE as isize - x
                };
                for t_x in 0..TerrainChunk::SQUARE as isize {
                    if terrain.get(t_x as usize, y as usize) > TerrainChunk::WATER_LEVEL {
                        let d = (t_x - x).abs();
                        if d < min_x_d {
                            min_x_d = d;
                            x_d = t_x - x;
                        }
                    }
                }
                let mut min_y_d = TerrainChunk::SQUARE as isize;
                let mut y_d = if y > TerrainChunk::SQUARE as isize / 2 {
                    -y
                } else {
                    TerrainChunk::SQUARE as isize - y
                };

                for t_y in 0..TerrainChunk::SQUARE as isize {
                    if terrain.get(x as usize, t_y as usize) > TerrainChunk::WATER_LEVEL {
                        let d = (t_y - y).abs();
                        if d < min_y_d {
                            min_y_d = d;
                            y_d = t_y - y;
                        }
                    }
                }
                w.set(x as usize, y as usize, IVec2::new(x_d as i32, y_d as i32));
            }
        }
        w
    }

    #[allow(dead_code)]
    pub fn format(&self) -> String {
        let mut s = String::new();
        for x in 0..TerrainChunk::SQUARE {
            for y in 0..TerrainChunk::SQUARE {
                let v = self.get(x, y);
                let v_x = v.x;
                let v_y = v.y;
                s.push_str(format!("|{v_x:02},{v_y:02}|").as_str());
            }
            s.push_str("\n");
        }
        s
    }

    /// assumes x and y 0..SQUARE
    pub fn set(&mut self, x: usize, y: usize, dir: IVec2) {
        assert!(x < TerrainChunk::SQUARE);
        assert!(y < TerrainChunk::SQUARE);

        let i = y * TerrainChunk::SQUARE + x;
        self.directions[i] = dir;
    }

    #[allow(dead_code)]
    pub fn get(&self, x: usize, y: usize) -> IVec2 {
        assert!(x < TerrainChunk::SQUARE);
        assert!(y < TerrainChunk::SQUARE);

        let i = y * TerrainChunk::SQUARE + x;
        self.directions[i]
    }
}

mod test {
    #[allow(unused)]
    use bevy::math::IVec2;

    #[allow(unused)]
    use crate::demo::terrain::{height::TerrainChunk, waves::Waves};

    #[test]
    fn waves_init() {
        let mut terrain = TerrainChunk::zero();
        for x in [0, TerrainChunk::SQUARE - 1] {
            for y in 0..TerrainChunk::SQUARE {
                terrain.set(x, y, 1.0);
            }
        }
        for y in [0, TerrainChunk::SQUARE - 1] {
            for x in 0..TerrainChunk::SQUARE {
                terrain.set(x, y, 1.0);
            }
        }

        let waves = Waves::init(&terrain);
        println!("{}", waves.format());

        assert_eq!(waves.get(0, 0), IVec2::ZERO);
        assert_eq!(waves.get(1, 0), IVec2::ZERO);
        assert_eq!(waves.get(0, 1), IVec2::ZERO);
        assert_eq!(waves.get(1, 1), IVec2::new(-1, -1)); // avg of |(1,1), (0,1)|,|(1,1), (1,0)|,
    }
}
