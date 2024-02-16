use glam::DVec3;

use crate::{
    bounding_box::BoundingBox, celestial_body::CelestialBody, simulation, z_order::z_order_curve,
};

/// When distance/radius < T, then we can do that Barnes-Hut optimisation
const T: f64 = 1.0;
const INV_T_SQUARED: f64 = 1.0 / (T * T);

pub struct CosmicSystem {
    pub bounding_box: BoundingBox,
    root: CosmicSystemNode,
}

impl CosmicSystem {
    pub fn new(bounding_box: BoundingBox, _capacity: usize) -> Self {
        Self {
            bounding_box,
            root: CosmicSystemNode::Empty,
        }
    }

    pub fn gravitational_force_zero_mass(&self, body: &CelestialBody) -> DVec3 {
        self.root.gravitational_force(body).unwrap_or_default() * simulation::G
    }

    pub fn add(&mut self, body: CelestialBody) {
        if !self.bounding_box.contains(body.position) {
            return;
        }
        let z_order = z_order_curve(body.position, &self.bounding_box);
        self.root
            .add(body, z_order, self.bounding_box.side_length());
    }
}

enum CosmicSystemNode {
    Empty,
    Leaf {
        value: CelestialBody,
        /// Remaining bits of the z-order
        z_order: u128,
    },
    Internal {
        value: CelestialBody,
        comparison_factor: f64,
        children: Box<[CosmicSystemNode; 8]>,
    },
}

impl CosmicSystemNode {
    pub fn new_leaf(value: CelestialBody, z_order: u128) -> Self {
        Self::Leaf { value, z_order }
    }

    pub fn new_internal(
        value: CelestialBody,
        side_length: f64,
        children: Box<[CosmicSystemNode; 8]>,
    ) -> Self {
        Self::Internal {
            value,
            comparison_factor: side_length * side_length * INV_T_SQUARED,
            children,
        }
    }

    pub fn gravitational_force(&self, body: &CelestialBody) -> Option<DVec3> {
        match self {
            CosmicSystemNode::Empty => None,
            CosmicSystemNode::Leaf { value, .. } => Some(body.gravitational_force_zero_mass(value)),
            CosmicSystemNode::Internal {
                value,
                comparison_factor,
                children,
            } => {
                // width / distance < T = Optimisation
                // width^2 / distance^2 < T^2 = Optimisation
                // width^2 < T^2 * distance^2 = Optimisation
                // width^2 * (1/T^2) < distance^2 = Optimisation
                if *comparison_factor < body.distance_to_squared(value) {
                    Some(body.gravitational_force_zero_mass(value))
                } else {
                    Some(
                        children
                            .iter()
                            .filter_map(|child| child.gravitational_force(body))
                            .sum(),
                    )
                }
            }
        }
    }

    pub fn add(&mut self, body: CelestialBody, z_order: u128, side_length: f64) {
        match self {
            CosmicSystemNode::Empty => *self = CosmicSystemNode::new_leaf(body, z_order),
            CosmicSystemNode::Leaf {
                value,
                z_order: mut existing_z_order,
            } => {
                // Leaf nodes get subdivided into internal nodes
                let new_value = CelestialBody::from_objects(value, &body);
                const ARRAY_REPEAT_VALUE: CosmicSystemNode = CosmicSystemNode::Empty;
                let mut new_children = Box::new([ARRAY_REPEAT_VALUE; 8]);

                let octant = get_octant(&mut existing_z_order);
                new_children[octant] = CosmicSystemNode::new_leaf(value.clone(), existing_z_order);

                *self = CosmicSystemNode::new_internal(new_value, side_length, new_children);

                // Add the new body
                self.add(body, z_order, side_length);
            }
            CosmicSystemNode::Internal {
                value, children, ..
            } => {
                *value = CelestialBody::from_objects(value, &body);

                // This is a node in the middle of our tree. We need to go deeper
                let mut z_order = z_order;
                let octant = get_octant(&mut z_order);
                children[octant].add(body, z_order, side_length / 2.0);
            }
        }
    }
}

#[inline]
fn get_octant(z_order: &mut u128) -> usize {
    // take most significant 3 bits
    let octant = *z_order >> 125;
    *z_order <<= 3;
    octant as usize
}
