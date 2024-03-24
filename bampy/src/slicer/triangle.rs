use approx::relative_eq;
use bvh::{
    aabb::{Aabb, Bounded},
    bounding_hierarchy::BHShape,
};
use nalgebra::{Point3, Vector3};

use super::{line::Line3, FloatValue};

#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub a: Vector3<FloatValue>,
    pub b: Vector3<FloatValue>,
    pub c: Vector3<FloatValue>,
    pub normal: Vector3<FloatValue>,
    node_index: usize,
    pub aabb: Aabb<FloatValue, 3>,
}

#[inline(always)]
fn vec_inside_aabb(vec: &Vector3<FloatValue>, aabb: &Aabb<FloatValue, 3>) -> bool {
    macro_rules! within {
        ($axis:ident) => {
            ((vec.$axis >= aabb.min.$axis && vec.$axis <= aabb.max.$axis)
                || relative_eq!(vec.$axis, aabb.min.$axis)
                || relative_eq!(vec.$axis, aabb.max.$axis))
        };
    }
    within!(x) && within!(y) && within!(z)
}

impl Triangle {
    pub fn new(a: Vector3<FloatValue>, b: Vector3<FloatValue>, c: Vector3<FloatValue>) -> Self {
        Self {
            a,
            b,
            c,
            normal: (b - a).cross(&(c - a)).into(),
            node_index: 0,
            aabb: Aabb::with_bounds(
                Point3::new(
                    FloatValue::min(FloatValue::max(a.x, b.x), c.x),
                    FloatValue::min(FloatValue::min(a.y, b.y), c.y),
                    FloatValue::min(FloatValue::min(a.z, b.z), c.z),
                ),
                Point3::new(
                    FloatValue::max(FloatValue::max(a.x, b.x), c.x),
                    FloatValue::max(FloatValue::max(a.y, b.y), c.y),
                    FloatValue::max(FloatValue::max(a.z, b.z), c.z),
                ),
            ),
        }
    }

    pub fn has_point_in_aabb(&self, aabb: &Aabb<FloatValue, 3>) -> bool {
        vec_inside_aabb(&self.a, aabb)
            || vec_inside_aabb(&self.b, aabb)
            || vec_inside_aabb(&self.c, aabb)
    }

    pub fn has_vec(&self, vec: Vector3<FloatValue>) -> bool {
        relative_eq!(self.a, vec) || relative_eq!(self.b, vec) || relative_eq!(self.c, vec)
    }

    pub fn shares_point_with_triangle(&self, other: Triangle) -> bool {
        self.has_vec(other.a) || self.has_vec(other.b) || self.has_vec(other.c)
    }

    pub fn shares_edge_with_triangle(&self, other: Triangle) -> bool {
        let a = self.has_vec(other.a);
        let b = self.has_vec(other.b);
        let c = self.has_vec(other.c);
        a && b || a && c || b && c
    }

    pub fn intersect_z(&self, z: FloatValue) -> Option<Line3> {
        let mut intersection = Vec::<Vector3<FloatValue>>::with_capacity(3);
        let mut last = &self.c;
        for point in [self.a, self.b, self.c].iter() {
            if relative_eq!(point.z, z) {
                intersection.push(Vector3::new(point.x, point.y, z));
            } else if last.z < z && point.z > z {
                let ratio = (z - last.z) / (point.z - last.z);
                intersection.push(Vector3::new(
                    last.x + (point.x - last.x) * ratio,
                    last.y + (point.y - last.y) * ratio,
                    z,
                ))
            } else if last.z > z && point.z < z {
                let ratio = (z - point.z) / (last.z - point.z);
                intersection.push(Vector3::new(
                    point.x + (last.x - point.x) * ratio,
                    point.y + (last.y - point.y) * ratio,
                    z,
                ))
            }
            last = point;
        }
        if intersection.len() == 2 {
            Some(Line3 {
                start: intersection[0],
                end: intersection[1],
            })
        } else {
            None
        }
    }
}

impl Bounded<FloatValue, 3> for Triangle {
    fn aabb(&self) -> Aabb<FloatValue, 3> {
        self.aabb
    }
}

impl BHShape<FloatValue, 3> for Triangle {
    fn set_bh_node_index(&mut self, node_index: usize) {
        self.node_index = node_index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}
