use glam::DVec3;

pub trait Vec3Extensions {
    fn all_less_than(&self, other: DVec3) -> bool;
}

impl Vec3Extensions for DVec3 {
    #[inline]
    fn all_less_than(&self, other: DVec3) -> bool {
        self.x < other.x && self.y < other.y && self.z < other.z
    }
}
