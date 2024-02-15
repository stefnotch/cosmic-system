use glam::DVec3;

use crate::bounding_box::BoundingBox;

pub fn z_order_curve(position: DVec3, bounding_box: BoundingBox) -> u128 {
    let relative_position = (position - bounding_box.min).max(DVec3::ZERO);
    let scaled_position = relative_position * (u32::MAX as f64 / bounding_box.side_length());
    let x = scaled_position.x as u32;
    let y = scaled_position.y as u32;
    let z = scaled_position.z as u32;
    // https://graphics.stanford.edu/~seander/bithacks.html#InterleaveTableObvious
    let mut result: u128 = 0;
    for i in 0u32..32 {
        let x_masked = (x & (1 << i)) as u128;
        let y_masked = (y & (1 << i)) as u128;
        let z_masked = (z & (1 << i)) as u128;

        result |= x_masked << (2 * i) | y_masked << (2 * i + 1) | z_masked << (2 * i + 2);
    }
    result << 32
}

// TODO: Write tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_z_order_curve() {
        let bounding_box = BoundingBox::new(DVec3::ZERO, DVec3::ONE);
        let position = DVec3::new(0., 0.5, 0.75);
        // 0 gets mapped to 0000..
        // 0.5 gets mapped to 0111.. (because we multiply it by 2^32 - 1)
        // 0.75 gets mapped to 1011..
        // so we should get something like 100 010 110 110 110 110 ...
        let result = z_order_curve(position, bounding_box);
        assert_eq!(result, 0b100010110110110110110110110110110110110110110110110110110110110110110110110110110110110110110110u128 << 32);
    }
}
