use glam::DVec3;

#[derive(Clone, Copy, Debug)]
pub struct CelestialBody {
    pub index: usize,
    pub position: DVec3,
    pub mass: f64,
    pub key: u128,

    pub current_movement: DVec3,
}

impl CelestialBody {
    pub fn new(index: usize, mass: f64, position: DVec3, current_movement: DVec3) -> Self {
        Self {
            index,
            mass,
            position,
            key: 0,
            current_movement,
        }
    }

    pub fn from_objects(a: &CelestialBody, b: &CelestialBody) -> CelestialBody {
        let mass = a.mass + b.mass;
        assert!(mass > 0.0);
        let center_of_mass = (a.position * (a.mass / mass)) + (b.position * (b.mass / mass));
        CelestialBody::new(0, mass, center_of_mass, DVec3::ZERO)
    }

    #[inline]
    pub fn distance_to_squared(&self, other: &CelestialBody) -> f64 {
        self.position.distance_squared(other.position)
    }

    /// Assume that self has zero mass.
    pub fn gravitational_force_zero_mass(&self, other: &CelestialBody) -> DVec3 {
        if self.key == other.key {
            return DVec3::ZERO;
        }

        let delta = other.position - self.position;
        let squared_distance = delta.length_squared();
        let force = other.mass / (squared_distance * squared_distance.sqrt());
        delta * force
    }

    pub fn add_force(&mut self, force: DVec3) {
        self.current_movement += force;
    }

    pub fn update(&mut self) {
        self.position += self.current_movement;
    }
}
