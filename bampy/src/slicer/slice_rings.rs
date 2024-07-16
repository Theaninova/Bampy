use approx::relative_eq;
use nalgebra::Vector3;

use crate::console_log;

use super::{
    base_slices::{create_base_slices, BaseSlice},
    mesh::Mesh,
    FloatValue, SlicerOptions,
};

#[derive(Debug)]
pub struct SliceRing {
    pub d: FloatValue,
    /// The points of the ring, in clockwise order.
    pub points: Vec<Vector3<FloatValue>>,
    pub closed: bool,
}

pub fn slice_rings<'a>(
    axis: usize,
    options: &'a SlicerOptions,
    slicable: &'a Mesh,
) -> impl Iterator<Item = Vec<SliceRing>> + 'a {
    let mut layer_index = 0;
    create_base_slices(axis, options, slicable)
        .map(move |slice| find_slice_rings(axis, slice, &mut layer_index))
}

pub fn find_slice_rings(
    axis: usize,
    mut slice: BaseSlice,
    layer_index: &mut u32,
) -> Vec<SliceRing> {
    let axis_a = (axis + 1) % 3;
    let axis_b = (axis + 2) % 3;
    let mut rings = vec![];
    while let Some(line) = slice.lines.pop() {
        if relative_eq!(line.start, line.end) {
            continue;
        }
        let mut right = vec![line.end];
        let mut left = vec![line.start];
        let mut previous_len = usize::MAX;
        let mut closed = false;

        while !closed {
            if previous_len == slice.lines.len() {
                break;
            }
            previous_len = slice.lines.len();

            slice.lines.retain_mut(|line| {
                if closed {
                    return true;
                }

                let test = |side: &mut Vec<Vector3<FloatValue>>| {
                    let last = side.last().unwrap();
                    let s = relative_eq!(line.start, last);
                    let e = relative_eq!(line.end, last);
                    if s && !e {
                        side.push(line.end);
                    } else if !s && e {
                        side.push(line.start);
                    }
                    s || e
                };

                if test(&mut left) || test(&mut right) {
                    closed = relative_eq!(left.last().unwrap(), right.last().unwrap());
                    false
                } else {
                    true
                }
            })
        }

        left.reverse();
        left.extend(right);
        let mut ring = SliceRing {
            d: slice.d,
            closed,
            points: left,
        };

        if ring.points.windows(2).fold(0.0, |acc, curr| {
            acc + (curr[1][axis_a] - curr[0][axis_a]) * (curr[1][axis_b] + curr[0][axis_b])
        }) < 0.0
        {
            ring.points.reverse();
        }

        rings.push(ring);
        *layer_index += 1;
    }

    rings
}
