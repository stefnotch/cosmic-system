use comfy::Vec3;

pub trait Vec3Extensions {
    fn all_less_than(&self, other: Vec3) -> bool;

    /**
     *
     * @return a 3 bit value encoding the correct octant relative to the center (self).
     */
    fn get_octant(&self, point: Vec3) -> usize;
}

impl Vec3Extensions for Vec3 {
    #[inline]
    fn all_less_than(&self, other: Vec3) -> bool {
        self.x < other.x && self.y < other.y && self.z < other.z
    }

    #[inline]
    fn get_octant(&self, point: Vec3) -> usize {
        let mut result = 0;
        if point.x >= self.x {
            result |= 1;
        }
        if point.y >= self.y {
            result |= 2;
        }
        if point.z >= self.z {
            result |= 4;
        }

        result
    }
}
