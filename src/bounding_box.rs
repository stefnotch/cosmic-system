use glam::DVec3;

use crate::vec3_extensions::Vec3Extensions;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundingBox {
    pub min: DVec3,
    pub max: DVec3,
}

impl BoundingBox {
    pub fn new(min: DVec3, max: DVec3) -> Self {
        Self { min, max }
    }

    pub fn contains(&self, point: DVec3) -> bool {
        !point.all_less_than(self.min) && point.all_less_than(self.max)
    }

    #[inline]
    pub fn side_length(&self) -> f64 {
        self.max.x - self.min.x
    }
}
