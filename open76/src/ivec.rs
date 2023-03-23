use std::ops::{Add, AddAssign};

use glam::{Vec3, vec3};

#[derive(Debug, Default, Clone, Copy)]
pub struct CellVec {
    pub cell_x: i32,
    pub cell_z: i32,
    pub delta: Vec3,
}

impl Add<Vec3> for CellVec {
    type Output = CellVec;

    fn add(self, rhs: Vec3) -> Self::Output {
        let mut new_delta = self.delta + rhs;
        let mut cell_x = self.cell_x;
        let mut cell_z = self.cell_z;

        while new_delta.x > 640.0 {
            new_delta.x -= 640.0;
            cell_x += 1;
        }

        while new_delta.z > 640.0 {
            new_delta.z -= 640.0;
            cell_z += 1;
        }

        while new_delta.x < 0.0 {
            new_delta.x += 640.0;
            cell_x -= 1;
        }

        while new_delta.z < 0.0 {
            new_delta.z += 640.0;
            cell_z -= 1;
        }

        Self::Output {
            cell_x,
            cell_z,
            delta: new_delta
        }
    }
}

impl AddAssign<Vec3> for CellVec {
    fn add_assign(&mut self, rhs: Vec3) {
        self.delta += rhs;

        while self.delta.x > 640.0 {
            self.delta.x -= 640.0;
            self.cell_x += 1;
        }

        while self.delta.z > 640.0 {
            self.delta.z -= 640.0;
            self.cell_z += 1;
        }

        while self.delta.x < 0.0 {
            self.delta.x += 640.0;
            self.cell_x -= 1;
        }

        while self.delta.z < 0.0 {
            self.delta.z += 640.0;
            self.cell_z -= 1;
        }
    }
}

impl From<Vec3> for CellVec {
    fn from(v: Vec3) -> Self {
        let cell_x = (v.x / 640.0) as i32;
        let cell_z = (v.z / 640.0) as i32;
        let delta = vec3(v.x % 640.0, v.y, v.z % 640.0);

        Self {
            cell_x,
            cell_z,
            delta
        }
    }
}

impl CellVec {
    pub fn absolute(&self) -> Vec3 {
        vec3((self.cell_x * 640) as f32, 0.0, (self.cell_z * 640) as f32) + self.delta
    }
}
