use glam::DVec3;

use crate::{bounding_box::BoundingBox, celestial_object::CelestialObject, simulation};

/// When distance/radius < T, then we can do that Barnes-Hut optimisation
const T: f64 = 1.0;
const T_SQUARED: f64 = T * T;

pub struct CosmicSystem {
    pub bounding_box: BoundingBox,
    root: Option<CosmicSystemNode>,
}

impl CosmicSystem {
    pub fn new(bounding_box: BoundingBox, _capacity: usize) -> Self {
        Self {
            bounding_box,
            root: None,
        }
    }

    pub fn gravitational_force(&self, body: &CelestialObject) -> DVec3 {
        match &self.root {
            Some(root) => root.gravitational_force(body) * simulation::G * body.mass,
            None => DVec3::ZERO,
        }
    }

    pub fn add(&mut self, body: CelestialObject) {
        if !self.bounding_box.contains(body.position) {
            return;
        }
        if let Some(root) = &mut self.root {
            root.add(body, self.bounding_box.clone());
        } else {
            self.root = Some(CosmicSystemNode::new(body, self.bounding_box.side_length()));
        }
    }
}

struct CosmicSystemNode {
    pub value: CelestialObject,
    pub side_length_squared: f64,
    /// References would be better
    pub children: Option<Box<[Option<CosmicSystemNode>; 8]>>,
}

impl CosmicSystemNode {
    pub fn new(value: CelestialObject, side_length: f64) -> Self {
        Self {
            value,
            side_length_squared: side_length * side_length,
            children: None,
        }
    }

    pub fn gravitational_force(&self, body: &CelestialObject) -> DVec3 {
        // < T = Optimisation
        if self.side_length_squared < T_SQUARED * body.distance_to_squared(&self.value) {
            return self.value.gravitational_force(body);
        }
        match &self.children {
            Some(children) => children
                .iter()
                .filter_map(|child| child.as_ref())
                .map(|child| child.gravitational_force(body))
                .sum(),
            None => self.value.gravitational_force(body),
        }
    }

    pub fn add(&mut self, body: CelestialObject, bounding_box: BoundingBox) {
        let old_value = self.value;

        // Compute the new celestial object (combined bodies)
        self.value = CelestialObject::from_objects(&self.value, &body);

        // Deal with leaves
        if self.children.is_none() {
            const ARRAY_REPEAT_VALUE: Option<CosmicSystemNode> = None;
            let mut new_children = Box::new([ARRAY_REPEAT_VALUE; 8]);
            let octant = bounding_box.get_octant(old_value.position);
            new_children[octant] = Some(CosmicSystemNode::new(
                old_value,
                bounding_box.side_length() * 0.5,
            ));

            self.children = Some(new_children);
        }

        // This is a node in the middle of our tree. We need to go deeper
        let octant = bounding_box.get_octant(body.position);
        let children = self.children.as_mut().unwrap();
        if let Some(child) = &mut children[octant] {
            child.add(body, bounding_box.get_octant_bounding_box(octant));
        } else {
            children[octant] = Some(CosmicSystemNode::new(
                body,
                bounding_box.side_length() * 0.5,
            ));
        }
    }
}
