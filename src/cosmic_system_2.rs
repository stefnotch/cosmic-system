use comfy::{IntoParallelRefMutIterator, ParallelIterator, ParallelSliceMut};
use glam::DVec3;

use crate::{
    bounding_box::BoundingBox, celestial_body::CelestialBody,
    celestial_body_extensions::CelestialBodyForces, simulation, z_order::z_order_curve,
};

/// When distance/radius < T, then we can do that Barnes-Hut optimisation
const T: f64 = 1.0;
const INV_T_SQUARED: f64 = 1.0 / (T * T);

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

    pub fn add_all(&mut self, bodies: &mut Vec<CelestialBody>) {
        bodies.par_iter_mut().for_each(|body| {
            body.key = z_order_curve(body.position, &self.bounding_box);
        });
        bodies.par_sort();

        // Clear nodes (just in case)
        for node in self.nodes.iter_mut() {
            *node = Default::default();
        }

        // We basically start in the middle.
        // All the bottom - 1 layer nodes come here
        // And then we go up to k / 2, where the bottom - 2 layer nodes come.
        let mut k = self.nodes.len() / 2;
        // We manually do the first iteration (bodies)
        for i in 0..k {
            let node_index = k + i;
            if 2 * i + 1 < bodies.len() {
                // Left and right bodies exist
                let left_body = &bodies[2 * i];
                let right_body = &bodies[2 * i + 1];
                let merged = CelestialBody::from_objects(left_body, right_body);
                self.nodes[node_index] = CosmicSystemNode {
                    position: merged.position,
                    mass: merged.mass,
                    comparison_factor: comparison_factor(
                        left_body.key,
                        right_body.key,
                        &self.bounding_box,
                    ),
                };
            } else if 2 * i < bodies.len() {
                // Only left body exists
                self.nodes[node_index] = CosmicSystemNode {
                    position: bodies[2 * i].position,
                    mass: bodies[2 * i].mass,
                    comparison_factor: -1.0,
                };
            } else {
                // No bodies exist
                self.nodes[node_index] = CosmicSystemNode {
                    position: DVec3::ZERO,
                    mass: 0.0,
                    comparison_factor: -1.0,
                };
            }
        }
        k /= 2;

        // And then the loop takes over (nodes)
        while k > 0 {
            for i in 0..k {
                let node_index = k + i;
                let left_node = &self.nodes[2 * node_index];
                let right_node = &self.nodes[2 * node_index + 1];

                if left_node.mass > 0.0 && right_node.mass > 0.0 {
                    // Left and right nodes truly exist
                    let merged = CelestialBody::from_objects(&left_node.body(), &right_node.body());
                    self.nodes[node_index] = CosmicSystemNode {
                        position: merged.position,
                        mass: merged.mass,
                        comparison_factor: todo!(), // TODO: left_node.comparison_factor * 2.0,
                    };
                } else if left_node.mass > 0.0 {
                    // Only left node truly exists
                    self.nodes[node_index] = CosmicSystemNode {
                        position: left_node.position,
                        mass: left_node.mass,
                        comparison_factor: left_node.comparison_factor,
                    };
                } else {
                    // No nodes actually exist
                    self.nodes[node_index] = CosmicSystemNode {
                        position: DVec3::ZERO,
                        mass: 0.0,
                        comparison_factor: -1.0,
                    };
                }
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
                let index = k - nodes.len();
                // Always valid indices, because a node always has 2 children
                // (If it only had one body as its child, then it would have a comparison_factor to -1, causing the function to return before getting here)
                return body.gravitational_force_zero_mass(&bodies[index])
                    + body.gravitational_force_zero_mass(&bodies[index + 1]);
            }

            let node = &nodes[k];
            let node_body = node.body();
            if node.comparison_factor < body.distance_to_squared(&node_body) {
                body.gravitational_force_zero_mass(&node_body)
            } else {
                helper(2 * k, body, nodes, bodies) + helper(2 * k + 1, body, nodes, bodies)
            }
        }

        helper(1, body, &self.nodes, bodies) * simulation::G
    }
}

/// Index of the bit where the z-orders differ
fn index_of_1(a: u128, b: u128) -> u8 {
    (a ^ b).leading_zeros() as u8
}

/// Side length for barnes hut
fn side_length(a: u128, b: u128, bounding_box: &BoundingBox) -> f64 {
    let number_of_splits = index_of_1(a, b);
    bounding_box.side_length() / ((1 << number_of_splits) as f64)
}

/// Comparison factor for barnes hut
fn comparison_factor(a: u128, b: u128, bounding_box: &BoundingBox) -> f64 {
    let side_length = side_length(a, b, bounding_box);
    side_length * side_length * INV_T_SQUARED
}

/// A node always has 2 children
/// The left child is at index 2 * i
/// The right child is at index 2 * i + 1
/// If it's a leaf node, then the comparison_factor is < 0
struct CosmicSystemNode {
    position: glam::DVec3,
    mass: f64,
    /// The two child nodes
    // children: [usize; 2],
    /// width / distance < T = Optimisation
    /// width^2 / distance^2 < T^2 = Optimisation
    /// width^2 < T^2 * distance^2 = Optimisation
    /// width^2 * (1/T^2) < distance^2 = Optimisation
    comparison_factor: f64,
}

impl CosmicSystemNode {
    #[inline]
    pub fn body(&self) -> CelestialBody {
        CelestialBody {
            position: self.position,
            mass: self.mass,
            key: 0,
        }
    }
}

impl Default for CosmicSystemNode {
    #[inline]
    fn default() -> Self {
        Self {
            position: glam::DVec3::ZERO,
            mass: 0.0,
            // children: [0, 0],
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
            glam::DVec3::ONE * -4.0 * simulation::AU,
            glam::DVec3::ONE * 4.0 * simulation::AU,
        );

        assert_eq!(
            side_length(0b1000 << 124, 0b0101 << 124, &bounding_box),
            bounding_box.side_length()
        );
        assert_eq!(
            side_length(0b1000 << 124, 0b1101 << 124, &bounding_box),
            bounding_box.side_length() / 2.0
        );
        assert_eq!(
            side_length(0b1000 << 124, 0b1001 << 124, &bounding_box),
            bounding_box.side_length() / 8.0
        );
    }
}
