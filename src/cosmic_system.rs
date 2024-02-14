use comfy::Vec3;

use crate::{bounding_box::BoundingBox, celestial_object::CelestialObject, simulation};

/// When distance/radius < T, then we can do that Barnes-Hut optimisation
const T: f32 = 1.0;
const T_SQUARED: f32 = T * T;

pub struct CosmicSystem {
    pub bounding_box: BoundingBox,
    pub root: Option<CosmicSystemNode>,
}

impl CosmicSystem {
    pub fn new(bounding_box: BoundingBox, _capacity: usize) -> Self {
        Self {
            bounding_box,
            root: None,
        }
    }

    pub fn gravitational_force(&self, body: &CelestialObject) -> Vec3 {
        match &self.root {
            Some(root) => root.gravitational_force(body) * simulation::G * body.mass,
            None => Vec3::ZERO,
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
    pub side_length_squared: f32,
    /// References would be better
    pub children: Option<Box<[Option<CosmicSystemNode>; 8]>>,
}

impl CosmicSystemNode {
    pub fn new(value: CelestialObject, side_length: f32) -> Self {
        Self {
            value,
            side_length_squared: side_length * side_length,
            children: None,
        }
    }

    pub fn gravitational_force(&self, body: &CelestialObject) -> Vec3 {
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

    /*
    public void add(CelestialObject celestialBody, BoundingBox box) {
        if(isLeaf()) {
            // This is a node at the bottom of our tree. Take this body and move it down as well
            CelestialObject thisCelestialBody = this.value;

            // TODO: Maybe catch the case where two bodies have the same paw-sition

            subdivide();

            int octant = box.getOctant(thisCelestialBody.getPosition());
            children[octant] = new CosmicSystemNode(thisCelestialBody, box.sideLength() * 0.5);
        }

        // Compute the celestial object (combined bodies)
        this.value = CelestialObject.fromObjects(this.value, celestialBody);

        // This is a node in the middle of our tree. We need to go deeper

        int octant = box.getOctant(celestialBody.getPosition());
        if(children[octant] == null) {
            children[octant] = new CosmicSystemNode(celestialBody, box.sideLength() * 0.5);
        } else {
            box.resizeToOctant(octant);
            children[octant].add(celestialBody, box);
        }
    } */
}
