use glam::DVec3;

use crate::{bounding_box::BoundingBox, celestial_object::CelestialObject, simulation};

/// When distance/radius < T, then we can do that Barnes-Hut optimisation
const T: f64 = 1.0;
const T_SQUARED: f64 = T * T;

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

    pub fn gravitational_force(&self, body: &CelestialObject) -> DVec3 {
        self.root.gravitational_force(body) * simulation::G * body.mass
    }

    pub fn add(&mut self, body: CelestialObject) {
        if !self.bounding_box.contains(body.position) {
            return;
        }
        self.root.add(body, self.bounding_box.clone());
    }
}

enum CosmicSystemNode {
    Empty,
    Leaf(CelestialObject),
    Internal {
        value: CelestialObject,
        side_length_squared: f64,
        children: Box<[CosmicSystemNode; 8]>,
    },
}

impl CosmicSystemNode {
    pub fn new_leaf(value: CelestialObject) -> Self {
        Self::Leaf(value)
    }

    pub fn new_internal(
        value: CelestialObject,
        side_length: f64,
        children: Box<[CosmicSystemNode; 8]>,
    ) -> Self {
        Self::Internal {
            value,
            side_length_squared: side_length * side_length,
            children,
        }
    }

    pub fn gravitational_force(&self, body: &CelestialObject) -> DVec3 {
        match self {
            CosmicSystemNode::Empty => DVec3::ZERO,
            CosmicSystemNode::Leaf(value) => body.gravitational_force(value),
            CosmicSystemNode::Internal {
                value,
                side_length_squared,
                children,
            } => {
                // < T = Optimisation
                if *side_length_squared < T_SQUARED * body.distance_to_squared(value) {
                    body.gravitational_force(value)
                } else {
                    children
                        .iter()
                        .map(|child| child.gravitational_force(body))
                        .sum()
                }
            }
        }
    }

    pub fn add(&mut self, body: CelestialObject, bounding_box: BoundingBox) {
        match self {
            CosmicSystemNode::Empty => *self = CosmicSystemNode::new_leaf(body),
            CosmicSystemNode::Leaf(value) => {
                // Leaf nodes get subdivided into internal nodes
                let new_value = CelestialObject::from_objects(value, &body);
                const ARRAY_REPEAT_VALUE: CosmicSystemNode = CosmicSystemNode::Empty;
                let mut new_children = Box::new([ARRAY_REPEAT_VALUE; 8]);

                let octant = bounding_box.get_octant(value.position);
                new_children[octant] = CosmicSystemNode::new_leaf(value.clone());

                *self = CosmicSystemNode::new_internal(
                    new_value,
                    bounding_box.side_length() * 0.5,
                    new_children,
                );

                // Add the new body
                self.add(body, bounding_box);
            }
            CosmicSystemNode::Internal {
                value, children, ..
            } => {
                *value = CelestialObject::from_objects(value, &body);

                // This is a node in the middle of our tree. We need to go deeper
                let octant = bounding_box.get_octant(body.position);
                children[octant].add(body, bounding_box.get_octant_bounding_box(octant));
            }
        }
    }
}
