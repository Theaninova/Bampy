use std::{collections::VecDeque, ops::RangeInclusive};

use approx::relative_eq;
use bvh::aabb::Aabb;
use nalgebra::Point3;

use super::{axis::Axis, mesh::Mesh, FloatValue};

#[derive(Debug, Default)]
pub struct SlicePath {
    pub i: usize,
    pub d: FloatValue,
    pub axis: Axis,
    /// The points of the ring, in clockwise order.
    pub points: Vec<Point3<FloatValue>>,
    pub closed: bool,
    pub aabb: Aabb<FloatValue, 3>,
}

pub struct SurfacePath {
    pub i: RangeInclusive<usize>,
    pub d: RangeInclusive<FloatValue>,
    pub axis: Axis,
    pub path: Vec<Point3<FloatValue>>,
    pub aabb: Aabb<FloatValue, 3>,
}

pub struct SurfacePathIterator {
    slices: Vec<Vec<SlicePath>>,
    axis: Axis,
    nozzle_width: FloatValue,
}

impl SurfacePathIterator {
    pub fn new(mesh: &Mesh, axis: Axis, nozzle_width: FloatValue) -> Self {
        let (h_axis, _) = axis.other();

        Self {
            slices: mesh
                .slice_paths(axis, nozzle_width)
                .map(|mut slice| {
                    for ring in &mut slice {
                        ring.points.sort_unstable_by(|a, b| {
                            a.coords[h_axis as usize]
                                .partial_cmp(&b.coords[h_axis as usize])
                                .unwrap()
                        })
                    }
                    slice.sort_unstable_by(|a, b| {
                        a.points.first().unwrap().coords[h_axis as usize]
                            .partial_cmp(&b.points.first().unwrap().coords[h_axis as usize])
                            .unwrap()
                    });
                    let mut iter = slice.into_iter();
                    let mut out = vec![iter.next().unwrap()];
                    for ring in iter {
                        if relative_eq!(
                            out.last().unwrap().points.last().unwrap(),
                            ring.points.first().unwrap(),
                            epsilon = nozzle_width
                        ) {
                            let element = out.last_mut().unwrap();
                            element.points.extend(ring.points);
                            element.aabb.join_mut(&ring.aabb);
                        } else {
                            out.push(ring);
                        }
                    }
                    out
                })
                .collect(),
            axis,
            nozzle_width,
        }
    }
}

fn squish(points: &mut Vec<Point3<FloatValue>>, axis: Axis, d: FloatValue) {
    macro_rules! ax {
        ($p: expr) => {
            $p.coords[axis as usize]
        };
    }
    let first = ax!(points.first().unwrap());
    let left = points.iter().position(|p| first - ax!(p) > d);
    let last = ax!(points.last().unwrap());
    let right = points.iter().rposition(|p| ax!(p) - last > d);

    if let (Some(left), Some(right)) = (left, right) {
        if left > right {
            // TODO
            return;
        }
        let total = ax!(points[left + 1]) - first;
        let delta = ax!(points[left]) - ax!(points[left + 1]);
        points.splice(
            0..left,
            vec![points[left].lerp(&points[left + 1], (d - total) / delta)],
        );

        let total = last - ax!(points[right + 1]);
        let delta = ax!(points[right + 1]) - ax!(points[right]);
    }
}

impl Iterator for SurfacePathIterator {
    type Item = SurfacePath;

    fn next(&mut self) -> Option<Self::Item> {
        self.slices.retain_mut(|slice| !slice.is_empty());

        let (h_axis, _) = self.axis.other();
        let mut iter = self.slices.iter_mut();

        let mut ring = iter.next()?.pop()?;
        // TODO: squish(&mut ring.points, h_axis, self.nozzle_width);
        let mut item = Self::Item {
            i: ring.i..=ring.i,
            d: ring.d..=ring.d,
            axis: ring.axis,
            aabb: ring.aabb,
            path: ring.points,
        };

        for slice in iter {
            if *item.i.end() != slice[0].i - 1 {
                break;
            }

            let last = item.path.last().unwrap();

            let mut d = FloatValue::MAX;
            let mut needs_reverse = false;
            let mut index = None;

            for (i, ring) in slice.iter().enumerate() {
                macro_rules! item {
                    ($a:ident, $metric: ident) => {
                        $a.aabb.$metric[h_axis as usize]
                    };
                }
                let a = item!(ring, max);
                let b = item!(item, min);
                if !(a > b || relative_eq!(a, b, epsilon = 0.1)) {
                    continue;
                }

                let a = item!(ring, min);
                let b = item!(item, max);
                if !(a < b || relative_eq!(a, b, epsilon = 0.1)) {
                    continue;
                }

                let d_left = last
                    .coords
                    .metric_distance(&ring.points.first().unwrap().coords);
                let d_right = last
                    .coords
                    .metric_distance(&ring.points.last().unwrap().coords);
                let d_min = d_left.min(d_right);
                if d_min < d {
                    d = d_min;
                    needs_reverse = d_left > d_right;
                    index = Some(i);
                }
            }

            if let Some(mut ring) = index.map(|i| slice.remove(i)) {
                // TODO: squish(&mut ring.points, h_axis, self.nozzle_width);
                if needs_reverse {
                    ring.points.reverse();
                }

                item.i = *item.i.start()..=ring.i;
                item.d = *item.d.start()..=ring.d;
                item.path.append(&mut ring.points);
                item.aabb.join_mut(&ring.aabb)
            }
        }

        Some(item)
    }
}
