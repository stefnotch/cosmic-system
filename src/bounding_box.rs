use glam::DVec3;

use crate::vec3_extensions::Vec3Extensions;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundingBox {
    pub min: DVec3,
    pub max: DVec3,

    // TODO: Maybe remove
    pub center: DVec3,
}

impl BoundingBox {
    pub fn new(min: DVec3, max: DVec3) -> Self {
        let center = (max + min) / 2.0;
        Self { min, max, center }
    }

    #[inline]
    pub fn contains(&self, point: DVec3) -> bool {
        !point.all_less_than(self.min) && point.all_less_than(self.max)
    }

    #[inline]
    pub fn get_octant(&self, point: DVec3) -> usize {
        self.center.get_octant(point)
    }

    #[inline]
    pub fn get_octant_bounding_box(&self, octant: usize) -> Self {
        let (start_x, end_x) = if (octant & 1) == 0 {
            (self.min.x, self.center.x)
        } else {
            (self.center.x, self.max.x)
        };

        let (start_y, end_y) = if (octant & 2) == 0 {
            (self.min.y, self.center.y)
        } else {
            (self.center.y, self.max.y)
        };

        let (start_z, end_z) = if (octant & 4) == 0 {
            (self.min.z, self.center.z)
        } else {
            (self.center.z, self.max.z)
        };

        Self::new(
            DVec3::new(start_x, start_y, start_z),
            DVec3::new(end_x, end_y, end_z),
        )
    }

    #[inline]
    pub fn side_length(&self) -> f64 {
        self.max.x - self.min.x
    }
}
