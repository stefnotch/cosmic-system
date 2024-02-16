use glam::DVec3;

// TODO: Undo the SoA transformation
#[derive(Clone, Copy, Debug)]
pub struct CelestialBody {
    pub position: DVec3,
    pub mass: f64,
    pub key: u128,
}

impl CelestialBody {
    pub fn new(mass: f64, position: DVec3) -> Self {
        Self {
            mass,
            position,
            key: 0,
        }
    }

    pub fn from_objects(a: &CelestialBody, b: &CelestialBody) -> CelestialBody {
        let mass = a.mass + b.mass;
        let center_of_mass = (a.position * (a.mass / mass)) + (b.position * (b.mass / mass));
        CelestialBody::new(mass, center_of_mass)
    }

    #[inline]
    pub fn distance_to_squared(&self, other: &CelestialBody) -> f64 {
        self.position.distance_squared(other.position)
    }

    /// Assume that self has zero mass.
    pub fn gravitational_force_zero_mass(&self, other: &CelestialBody) -> DVec3 {
        let delta = other.position - self.position;
        let squared_distance = delta.length_squared();
        if squared_distance == 0.0 {
            return DVec3::ZERO;
        }
        let force = other.mass / (squared_distance * squared_distance.sqrt());
        delta * force
    }
}

impl PartialEq for CelestialBody {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for CelestialBody {}

impl Ord for CelestialBody {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

impl PartialOrd for CelestialBody {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
