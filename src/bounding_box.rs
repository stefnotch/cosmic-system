use comfy::Vec3;

use crate::vec3_extensions::Vec3Extensions;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,

    // TODO: Maybe remove
    pub center: Vec3,
}

impl BoundingBox {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        let center = (max + min) / 2.0;
        Self { min, max, center }
    }

    pub fn contains(&self, point: Vec3) -> bool {
        !point.all_less_than(self.min) && point.all_less_than(self.max)
    }

    pub fn get_octant(&self, point: Vec3) -> usize {
        self.center.get_octant(point)
    }

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
            Vec3::new(start_x, start_y, start_z),
            Vec3::new(end_x, end_y, end_z),
        )
    }

    pub fn side_length(&self) -> f32 {
        self.max.x - self.min.x
    }
}
