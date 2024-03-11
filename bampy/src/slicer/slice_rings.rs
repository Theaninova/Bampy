use approx::{relative_eq, relative_ne};
use nalgebra::Vector3;

use crate::console_log;

use super::base_slices::BaseSlice;

#[derive(Debug)]
pub struct SliceRing {
    pub z: f64,
    pub points: Vec<Vector3<f64>>,
}

pub fn find_slice_rings(mut slice: BaseSlice) -> Vec<SliceRing> {
    let mut rings = vec![];
    while let Some(line) = slice.lines.pop() {
        if relative_eq!(line.start, line.end) {
            continue;
        }
        let mut ring = SliceRing {
            z: slice.z,
            points: vec![line.start, line.end],
        };
        let mut right_start = ring.points[0];
        let mut right = ring.points[1];

        let mut previous_len = usize::MAX;
        while relative_ne!(ring.points[0], right) {
            if previous_len == slice.lines.len() {
                console_log!(
                    "Error: Could not find a ring for slice at z = {}, d = {}, {} items left.",
                    slice.z,
                    ring.points[0].metric_distance(&right),
                    slice.lines.len()
                );
                break;
            }
            previous_len = slice.lines.len();

            slice.lines.retain_mut(|line| {
                let s = relative_eq!(line.start, right);
                let e = relative_eq!(line.end, right);
                if s && !e && !relative_eq!(line.end, right_start) {
                    ring.points.push(line.end);
                    right_start = right;
                    right = line.end;
                    false
                } else if e && !s && !relative_eq!(line.start, right_start) {
                    ring.points.push(line.start);
                    right_start = right;
                    right = line.start;
                    false
                } else {
                    true
                }
            })
        }
        rings.push(ring)
    }

    rings
}
