use approx::relative_eq;
use nalgebra::Vector3;

use crate::console_log;

use super::{base_slices::BaseSlice, FloatValue};

#[derive(Debug)]
pub struct SliceRing {
    pub z: FloatValue,
    /// The points of the ring, in clockwise order.
    pub points: Vec<Vector3<FloatValue>>,
}

pub fn find_slice_rings(mut slice: BaseSlice, layer_index: &mut u32) -> Vec<SliceRing> {
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
        let mut sum_of_edges = (right.x - right_start.x) * (right.y + right_start.y);

        let mut previous_len = usize::MAX;
        let mut done = false;

        while !done {
            if previous_len == slice.lines.len() {
                console_log!(
                    "Error: Could not close ring {}, d = {}, {} items left.",
                    layer_index,
                    ring.points[0].metric_distance(&right),
                    slice.lines.len()
                );
                break;
            }
            previous_len = slice.lines.len();

            slice.lines.retain_mut(|line| {
                if done {
                    return true;
                }

                macro_rules! add {
                    ( $point:expr ) => {
                        if !relative_eq!($point, right_start) {
                            right_start = right;
                            right = $point;
                            ring.points.push(right);
                            sum_of_edges = (right.x - right_start.x) * (right.y + right_start.y);
                            done = relative_eq!(ring.points[0], right);
                        }
                    };
                }

                let s = relative_eq!(line.start, right);
                let e = relative_eq!(line.end, right);
                if s && !e {
                    add!(line.end);
                    false
                } else if e && !s {
                    add!(line.start);
                    false
                } else {
                    true
                }
            })
        }

        // The end point is duplicate, so not part of the winding order calculation.
        if sum_of_edges < 0.0 {
            ring.points.reverse();
        }
        rings.push(ring);
        *layer_index += 1;
    }

    rings
}
