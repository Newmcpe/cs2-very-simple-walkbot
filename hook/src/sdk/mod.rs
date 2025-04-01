use std::ops::Sub;

pub mod client_dll;
pub mod user_cmd;

#[derive(Debug, Clone)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Sub for Vector3 {
    type Output = Vector3;

    fn sub(self, rhs: Self) -> Self::Output {
        let delta = Vector3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        };
        delta
    }
}

impl Vector3 {
    pub fn distance2d(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

#[derive(Debug)]
pub struct QAngle {
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}
