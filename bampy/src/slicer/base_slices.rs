use super::{aabb_from_points, axis::Axis, line::Line3, slice_path::SlicePath, FloatValue};
use approx::relative_eq;
use nalgebra::Point3;

#[derive(Debug)]
pub struct BaseSlice {
    pub i: usize,
    pub d: FloatValue,
    pub axis: Axis,
    pub lines: Vec<Line3>,
}

impl BaseSlice {
    pub fn find_paths(mut self) -> Vec<SlicePath> {
        let (axis_a, axis_b) = self.axis.other();
        let mut rings = vec![];
        while let Some(line) = self.lines.pop() {
            if relative_eq!(line.start, line.end) {
                continue;
            }
            let mut right = vec![line.end];
            let mut left = vec![line.start];
            let mut previous_len = usize::MAX;
            let mut closed = false;

            while !closed {
                if previous_len == self.lines.len() {
                    break;
                }
                previous_len = self.lines.len();

                self.lines.retain_mut(|line| {
                    if closed {
                        return true;
                    }

                    let test = |side: &mut Vec<Point3<FloatValue>>| {
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
            let mut ring = SlicePath {
                d: self.d,
                i: self.i,
                axis: self.axis,
                closed,
                aabb: aabb_from_points(left.iter()),
                points: left,
            };

            if ring.points.windows(2).fold(0.0, |acc, curr| {
                acc + (curr[1][axis_a as usize] - curr[0][axis_a as usize])
                    * (curr[1][axis_b as usize] + curr[0][axis_b as usize])
            }) < 0.0
            {
                ring.points.reverse();
            }

            rings.push(ring);
        }

        rings
    }
}
