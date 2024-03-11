use approx::{relative_eq, relative_ne};
use nalgebra::Vector3;

use crate::console_log;

use super::base_slices::BaseSlice;

#[derive(Debug)]
pub struct SliceRing {
    pub z: f32,
    pub points: Vec<Vector3<f32>>,
}

pub fn find_slice_rings(mut slice: BaseSlice) -> Vec<SliceRing> {
    let mut rings = vec![];
    while let Some(line) = slice.lines.pop() {
        let mut ring = SliceRing {
            z: slice.z,
            points: vec![line.start, line.end],
        };
        let mut right = ring.points[1];

        let mut previous_len = usize::MAX;
        while relative_ne!(ring.points[0], right) {
            if previous_len == slice.lines.len() {
                console_log!(
                    "Error: Could not find a ring for slice at z = {}, {} items left.",
                    slice.z,
                    slice.lines.len()
                );
                break;
            }
            previous_len = slice.lines.len();

            slice.lines.retain_mut(|line| {
                //if relative_eq!(line.start, right, epsilon = 0.001) {
                ring.points.push(line.start);
                ring.points.push(line.end);
                right = line.end;
                false
                /*} else if relative_eq!(line.end, right, epsilon = 0.001) {
                    ring.points.push(line.start);
                    right = line.start;
                    false
                } else {
                    true
                }*/
            })
        }

        rings.push(ring)
    }

    rings
}
