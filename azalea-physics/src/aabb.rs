use crate::BlockHitResult;
use azalea_core::{Axis, BlockPos, Direction, PositionXYZ, Vec3};

pub const EPSILON: f64 = 1.0E-7;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AABB {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,

    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

impl AABB {
    pub fn contract(&self, x: f64, y: f64, z: f64) -> AABB {
        let mut min_x = self.min_x;
        let mut min_y = self.min_y;
        let mut min_z = self.min_z;

        let mut max_x = self.max_x;
        let mut max_y = self.max_y;
        let mut max_z = self.max_z;

        if x < 0.0 {
            min_x -= x;
        } else if x > 0.0 {
            max_x -= x;
        }

        if y < 0.0 {
            min_y -= y;
        } else if y > 0.0 {
            max_y -= y;
        }

        if z < 0.0 {
            min_z -= z;
        } else if z > 0.0 {
            max_z -= z;
        }

        AABB {
            min_x,
            min_y,
            min_z,

            max_x,
            max_y,
            max_z,
        }
    }

    pub fn expand_towards(&self, other: &Vec3) -> AABB {
        let mut min_x = self.min_x;
        let mut min_y = self.min_y;
        let mut min_z = self.min_z;

        let mut max_x = self.max_x;
        let mut max_y = self.max_y;
        let mut max_z = self.max_z;

        if other.x < 0.0 {
            min_x += other.x;
        } else if other.x > 0.0 {
            max_x += other.x;
        }

        if other.y < 0.0 {
            min_y += other.y;
        } else if other.y > 0.0 {
            max_y += other.y;
        }

        if other.z < 0.0 {
            min_z += other.z;
        } else if other.z > 0.0 {
            max_z += other.z;
        }

        AABB {
            min_x,
            min_y,
            min_z,

            max_x,
            max_y,
            max_z,
        }
    }

    pub fn inflate(&self, x: f64, y: f64, z: f64) -> AABB {
        let min_x = self.min_x - x;
        let min_y = self.min_y - y;
        let min_z = self.min_z - z;

        let max_x = self.max_x + x;
        let max_y = self.max_y + y;
        let max_z = self.max_z + z;

        AABB {
            min_x,
            min_y,
            min_z,

            max_x,
            max_y,
            max_z,
        }
    }

    pub fn intersect(&self, other: &AABB) -> AABB {
        let min_x = self.min_x.max(other.min_x);
        let min_y = self.min_y.max(other.min_y);
        let min_z = self.min_z.max(other.min_z);

        let max_x = self.max_x.min(other.max_x);
        let max_y = self.max_y.min(other.max_y);
        let max_z = self.max_z.min(other.max_z);

        AABB {
            min_x,
            min_y,
            min_z,

            max_x,
            max_y,
            max_z,
        }
    }

    pub fn minmax(&self, other: &AABB) -> AABB {
        let min_x = self.min_x.min(other.min_x);
        let min_y = self.min_y.min(other.min_y);
        let min_z = self.min_z.min(other.min_z);

        let max_x = self.max_x.max(other.max_x);
        let max_y = self.max_y.max(other.max_y);
        let max_z = self.max_z.max(other.max_z);

        AABB {
            min_x,
            min_y,
            min_z,

            max_x,
            max_y,
            max_z,
        }
    }

    pub fn move_relative(&self, x: f64, y: f64, z: f64) -> AABB {
        AABB {
            min_x: self.min_x + x,
            min_y: self.min_y + y,
            min_z: self.min_z + z,

            max_x: self.max_x + x,
            max_y: self.max_y + y,
            max_z: self.max_z + z,
        }
    }

    pub fn intersects_aabb(&self, other: &AABB) -> bool {
        self.min_x < other.max_x
            && self.max_x > other.min_x
            && self.min_y < other.max_y
            && self.max_y > other.min_y
            && self.min_z < other.max_z
            && self.max_z > other.min_z
    }
    pub fn intersects_vec3(&self, other: &Vec3, other2: &Vec3) -> bool {
        self.intersects_aabb(&AABB {
            min_x: other.x.min(other2.x),
            min_y: other.y.min(other2.y),
            min_z: other.z.min(other2.z),

            max_x: other.x.max(other2.x),
            max_y: other.y.max(other2.y),
            max_z: other.z.max(other2.z),
        })
    }

    pub fn contains(&self, x: f64, y: f64, z: f64) -> bool {
        x >= self.min_x
            && x < self.max_x
            && y >= self.min_y
            && y < self.max_y
            && z >= self.min_z
            && z < self.max_z
    }

    pub fn size(&self) -> f64 {
        let x = self.get_size(Axis::X);
        let y = self.get_size(Axis::Y);
        let z = self.get_size(Axis::Z);
        (x + y + z) / 3.0
    }

    pub fn get_size(&self, axis: Axis) -> f64 {
        axis.choose(
            self.max_x - self.min_x,
            self.max_y - self.min_y,
            self.max_z - self.min_z,
        )
    }

    pub fn deflate(&mut self, x: f64, y: f64, z: f64) -> AABB {
        self.inflate(-x, -y, -z)
    }

    pub fn clip(&self, min: &Vec3, max: &Vec3) -> Option<Vec3> {
        let mut t = [1.0];
        let x = max.x - min.x;
        let y = max.y - min.y;
        let z = max.z - min.z;
        let dir = self.get_direction(self, min, &mut t, None, x, y, z);
        if dir.is_none() {
            return None;
        }
        let t = t[0];
        Some(min.add(t * x, t * y, t * z))
    }

    pub fn clip_iterable(
        &self,
        boxes: &Vec<AABB>,
        from: &Vec3,
        to: &Vec3,
        pos: &BlockPos,
    ) -> Option<BlockHitResult> {
        let mut t = [1.0];
        let mut dir = None;
        let x = to.x - from.x;
        let y = to.y - from.y;
        let z = to.z - from.z;

        for aabb in boxes {
            dir = self.get_direction(aabb, from, &mut t, dir, x, y, z);
        }
        if dir.is_none() {
            return None;
        }
        let t = t[0];
        Some(BlockHitResult {
            location: from.add(t * x, t * y, t * z),
            direction: dir.unwrap(),
            block_pos: *pos,
            inside: false,
            miss: false,
        })
    }

    fn get_direction(
        &self,
        aabb: &AABB,
        from: &Vec3,
        t: &mut [f64],
        dir: Option<Direction>,
        x: f64,
        y: f64,
        z: f64,
    ) -> Option<Direction> {
        if x > EPSILON {
            return self.clip_point(
                t,
                dir,
                x,
                y,
                z,
                aabb.min_x,
                aabb.min_y,
                aabb.max_y,
                aabb.min_z,
                aabb.max_z,
                Direction::West,
                from.x,
                from.y,
                from.z,
            );
        } else if x < -EPSILON {
            return self.clip_point(
                t,
                dir,
                x,
                y,
                z,
                aabb.max_x,
                aabb.min_y,
                aabb.max_y,
                aabb.min_z,
                aabb.max_z,
                Direction::East,
                from.x,
                from.y,
                from.z,
            );
        }

        if y > EPSILON {
            return self.clip_point(
                t,
                dir,
                y,
                z,
                x,
                aabb.min_y,
                aabb.min_z,
                aabb.max_z,
                aabb.min_x,
                aabb.max_x,
                Direction::Down,
                from.y,
                from.z,
                from.x,
            );
        } else if y < -EPSILON {
            return self.clip_point(
                t,
                dir,
                y,
                z,
                x,
                aabb.max_y,
                aabb.min_z,
                aabb.max_z,
                aabb.min_x,
                aabb.max_x,
                Direction::Up,
                from.y,
                from.z,
                from.x,
            );
        }

        if z > EPSILON {
            return self.clip_point(
                t,
                dir,
                z,
                x,
                y,
                aabb.min_z,
                aabb.min_x,
                aabb.max_x,
                aabb.min_y,
                aabb.max_y,
                Direction::North,
                from.z,
                from.x,
                from.y,
            );
        } else if z < -EPSILON {
            return self.clip_point(
                t,
                dir,
                z,
                x,
                y,
                aabb.max_z,
                aabb.min_x,
                aabb.max_x,
                aabb.min_y,
                aabb.max_y,
                Direction::South,
                from.z,
                from.x,
                from.y,
            );
        }

        dir
    }

    fn clip_point(
        &self,
        t: &mut [f64],
        approach_dir: Option<Direction>,
        delta_x: f64,
        delta_y: f64,
        delta_z: f64,
        begin: f64,
        min_x: f64,
        max_x: f64,
        min_z: f64,
        max_z: f64,
        result_dir: Direction,
        start_x: f64,
        start_y: f64,
        start_z: f64,
    ) -> Option<Direction> {
        let t_x = (begin - start_x) / delta_x;
        let t_y = (start_y + t_x) / delta_y;
        let t_z = (start_z + t_x) / delta_z;
        if 0.0 < t_x
            && t_x < t[0]
            && min_x - EPSILON < t_y
            && t_y < max_x + EPSILON
            && min_z - EPSILON < t_z
            && t_z < max_z + EPSILON
        {
            t[0] = t_x;
            Some(result_dir)
        } else {
            approach_dir
        }
    }

    pub fn has_nan(&self) -> bool {
        self.min_x.is_nan()
            || self.min_y.is_nan()
            || self.min_z.is_nan()
            || self.max_x.is_nan()
            || self.max_y.is_nan()
            || self.max_z.is_nan()
    }

    pub fn get_center(&self) -> Vec3 {
        Vec3 {
            x: (self.min_x + self.max_x) / 2.0,
            y: (self.min_y + self.max_y) / 2.0,
            z: (self.min_z + self.max_z) / 2.0,
        }
    }

    pub fn of_size(center: Vec3, dx: f64, dy: f64, dz: f64) -> AABB {
        AABB {
            min_x: center.x - dx / 2.0,
            min_y: center.y - dy / 2.0,
            min_z: center.z - dz / 2.0,
            max_x: center.x + dx / 2.0,
            max_y: center.y + dy / 2.0,
            max_z: center.z + dz / 2.0,
        }
    }

    pub fn max(&self, axis: &Axis) -> f64 {
        axis.choose(self.max_x, self.max_y, self.max_z)
    }
    pub fn min(&self, axis: &Axis) -> f64 {
        axis.choose(self.min_x, self.min_y, self.min_z)
    }
}
