use comfy::{IntoParallelRefMutIterator, ParallelIterator, ParallelSliceMut};
use glam::DVec3;

use crate::{
    bounding_box::BoundingBox, celestial_body::CelestialBody, simulation, z_order::z_order_curve,
};

/// When distance/radius < T, then we can do that Barnes-Hut optimisation
const T: f64 = 1.0;
const INV_T_SQUARED: f64 = 1.0 / (T * T);

#[derive(Clone)]
pub struct CosmicSystem {
    bounding_box: BoundingBox,
    /// Binary search tree nodes.
    /// The root node is at index 1.
    /// Always a power of 2 size.
    /// See https://algorithmica.org/en/eytzinger
    nodes: Vec<CosmicSystemNode>,
}

impl CosmicSystem {
    pub fn new(bounding_box: BoundingBox, capacity: usize) -> Self {
        let capacity = capacity.next_power_of_two();
        let mut nodes = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            nodes.push(Default::default());
        }
        Self {
            bounding_box,
            nodes,
        }
    }

    pub fn set_all(&mut self, bodies: &mut Vec<CelestialBody>) {
        bodies.par_iter_mut().for_each(|body| {
            body.key = z_order_curve(body.position, &self.bounding_box);
        });
        bodies.par_sort_by_key(|body| body.key);

        assert!(bodies.len() <= self.nodes.len());

        // We basically start in the middle.
        // All the bottom - 1 layer nodes come here
        // And then we go up to k / 2, where the bottom - 2 layer nodes come.
        let mut k = self.nodes.len() / 2;
        // We manually do the first iteration (bodies)
        for i in 0..k {
            let node_index = k + i;
            self.nodes[node_index] = if 2 * i + 1 < bodies.len() {
                // Left and right bodies exist
                let left_body = &bodies[2 * i];
                let right_body = &bodies[2 * i + 1];
                let index_of_1 = index_of_1(left_body.key, right_body.key);
                CosmicSystemNode::from_bodies(left_body, right_body, index_of_1, &self.bounding_box)
            } else if 2 * i < bodies.len() {
                // Only left body exists
                CosmicSystemNode {
                    position: bodies[2 * i].position,
                    mass: bodies[2 * i].mass,
                    z_order: bodies[2 * i].key,
                    index_of_1: u8::MAX,
                    comparison_factor: -1.0,
                }
            } else {
                // No bodies exist
                Default::default()
            }
        }
        k /= 2;

        // And then the loop takes over (nodes)
        while k > 0 {
            for i in 0..k {
                let node_index = k + i;
                let left_node = &self.nodes[2 * node_index];
                let right_node = &self.nodes[2 * node_index + 1];

                self.nodes[node_index] = if left_node.mass > 0.0 && right_node.mass > 0.0 {
                    let index_of_1 = index_of_1(left_node.z_order, right_node.z_order)
                        .min(left_node.index_of_1.min(right_node.index_of_1));

                    CosmicSystemNode::from_bodies(
                        &left_node.body(),
                        &right_node.body(),
                        index_of_1,
                        &self.bounding_box,
                    )
                } else if left_node.mass > 0.0 {
                    // Only left node truly exists
                    let mut node = std::mem::take(&mut self.nodes[2 * node_index]);
                    node.comparison_factor = -1.0;
                    node
                } else {
                    assert!(right_node.mass <= 0.0);
                    // No nodes actually exist
                    Default::default()
                };
            }

            k /= 2;
        }
    }

    pub fn gravitational_force_zero_mass(
        &self,
        body: &CelestialBody,
        bodies: &Vec<CelestialBody>,
    ) -> DVec3 {
        fn helper(
            k: usize,
            body: &CelestialBody,
            nodes: &Vec<CosmicSystemNode>,
            bodies: &Vec<CelestialBody>,
        ) -> DVec3 {
            if k >= nodes.len() {
                // We're querying a single body itself
                let index = k - nodes.len();
                return body.gravitational_force_zero_mass(&bodies[index]);
            }

            let node = &nodes[k];
            let node_body = node.body();
            assert!(node.mass > 0.0);

            if node.comparison_factor < body.distance_to_squared(&node_body) {
                body.gravitational_force_zero_mass(&node_body)
            } else if node.comparison_factor < 0.0 {
                body.gravitational_force_zero_mass(&node_body)
            } else {
                assert!(node.comparison_factor >= 0.0);
                // Always valid indices, because a node always has 2 children
                // (If it only had one body as its child, then it would have a comparison_factor to -1, causing the function to return before getting here)
                helper(2 * k, body, nodes, bodies) + helper(2 * k + 1, body, nodes, bodies)
            }
        }

        helper(1, body, &self.nodes, bodies) * simulation::G
    }
}

/// Index of the bit where the z-orders differ
fn index_of_1(a: u128, b: u128) -> u8 {
    let index_of_1 = (a ^ b).leading_zeros() as u8;
    if index_of_1 < 128 {
        index_of_1
    } else {
        u8::MAX
    }
}

/// Side length for barnes hut
fn side_length(number_of_splits: u8, bounding_box: &BoundingBox) -> f64 {
    let number_of_cube_splits = number_of_splits / 3;
    bounding_box.side_length() / ((1u128 << number_of_cube_splits) as f64)
}

/// Comparison factor for barnes hut
fn comparison_factor(number_of_splits: u8, bounding_box: &BoundingBox) -> f64 {
    if number_of_splits == u8::MAX {
        // Special case where the nodes have the same key
        return -1.0;
    }
    assert!(
        number_of_splits < 128,
        "number_of_splits: {}",
        number_of_splits
    );
    let side_length = side_length(number_of_splits, bounding_box);
    side_length * side_length * INV_T_SQUARED
}

/// A node always has 2 children
/// The left child is at index 2 * i
/// The right child is at index 2 * i + 1
/// If it's a leaf node, then the comparison_factor is < 0
#[derive(Clone, Debug)]
struct CosmicSystemNode {
    position: DVec3,
    mass: f64,
    /// The two child nodes
    // children: [usize; 2],
    /// width / distance < T = Optimisation
    /// width^2 / distance^2 < T^2 = Optimisation
    /// width^2 < T^2 * distance^2 = Optimisation
    /// width^2 * (1/T^2) < distance^2 = Optimisation
    comparison_factor: f64,

    z_order: u128,
    index_of_1: u8,
}

const NEVER_KEY: u128 = u128::MAX;

impl CosmicSystemNode {
    #[inline]
    pub fn body(&self) -> CelestialBody {
        CelestialBody {
            position: self.position,
            mass: self.mass,
            key: self.z_order,
            current_movement: DVec3::ZERO,
        }
    }

    pub fn from_bodies(
        a: &CelestialBody,
        b: &CelestialBody,
        index_of_1: u8,
        bounding_box: &BoundingBox,
    ) -> Self {
        let mass = a.mass + b.mass;
        assert!(mass > 0.0);
        let position = (a.position * (a.mass / mass)) + (b.position * (b.mass / mass));
        let key = if index_of_1 == u8::MAX {
            // The children all have the same key
            a.key
        } else {
            // We want inner nodes to have a key that's never equal to a body's key
            NEVER_KEY
        };
        let merged = CelestialBody {
            mass,
            position,
            key,
            current_movement: DVec3::ZERO,
        };

        // If nodes have the same key, then index_of_1 is u8::MAX, which the comparison_factor function handles
        CosmicSystemNode {
            position: merged.position,
            mass: merged.mass,
            z_order: merged.key,
            index_of_1,
            comparison_factor: comparison_factor(index_of_1, &bounding_box),
        }
    }
}

impl Default for CosmicSystemNode {
    #[inline]
    fn default() -> Self {
        Self {
            position: DVec3::ZERO,
            mass: 0.0,
            z_order: 0,
            index_of_1: u8::MAX,
            comparison_factor: -1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::simulation;

    use super::*;

    /// Test the side_length
    #[test]
    fn test_side_length() {
        let bounding_box = BoundingBox::new(
            DVec3::ONE * -4.0 * simulation::AU,
            DVec3::ONE * 4.0 * simulation::AU,
        );

        assert_eq!(
            side_length(index_of_1(0b1000 << 124, 0b0101 << 124), &bounding_box),
            bounding_box.side_length()
        );
        assert_eq!(
            side_length(index_of_1(0b1000 << 124, 0b1101 << 124), &bounding_box),
            bounding_box.side_length()
        );
        assert_eq!(
            side_length(index_of_1(0b1000 << 124, 0b1001 << 124), &bounding_box),
            bounding_box.side_length() / 2.0
        );
    }
    /*
    #[test]
    fn test_with_equally_spaced_bodies() {
        let bounding_box = BoundingBox::new(DVec3::ONE * -100.0, DVec3::ONE * 100.0);
        let mut bodies = vec![
            CelestialBody::new(1.0, DVec3::new(10.0, 0.0, 0.0)),
            // CelestialBody::new(1.0, DVec3::new(0.0, 0.0, 0.0)),
            CelestialBody::new(2.0, DVec3::new(-10.0, 0.0, 0.0)),
            // CelestialBody::new(1.0, DVec3::new(20.0, 0.0, 0.0)),
            // CelestialBody::new(1.0, DVec3::new(30.0, 0.0, 0.0)),
        ];

        let mut cosmic_system = CosmicSystem::new(bounding_box, bodies.len());
        cosmic_system.set_all(&mut bodies);
        println!("{:#?}", bodies);

        println!("{:#?}", cosmic_system.nodes);

        let force = cosmic_system.gravitational_force_zero_mass(&bodies[0], &bodies);
        println!("{:#?}", force);
    } */
}
