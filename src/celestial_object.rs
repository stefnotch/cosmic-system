use glam::DVec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CelestialObject {
    pub mass: f64,
    pub position: DVec3,
}

impl CelestialObject {
    pub fn new(mass: f64, position: DVec3) -> Self {
        Self { mass, position }
    }

    pub fn from_objects(a: &CelestialObject, b: &CelestialObject) -> CelestialObject {
        let mass = a.mass + b.mass;
        let center_of_mass = (a.position * (a.mass / mass)) + (b.position * (b.mass / mass));
        CelestialObject::new(mass, center_of_mass)
    }

    #[inline]
    pub fn distance_to_squared(&self, other: &CelestialObject) -> f64 {
        self.position.distance_squared(other.position)
    }

    /// Assume that self has zero mass.
    pub fn gravitational_force_zero_mass(&self, other: &CelestialObject) -> DVec3 {
        let delta = other.position - self.position;
        let squared_distance = delta.length_squared();
        if squared_distance == 0.0 {
            return DVec3::ZERO;
        }
        let force = other.mass / (squared_distance * squared_distance.sqrt());
        delta * force
    }
}
