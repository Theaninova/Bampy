use approx::relative_eq;
use bvh::{
    aabb::{Aabb, Bounded},
    bounding_hierarchy::BHShape,
};
use nalgebra::{Point3, Vector3};

use super::{line::Line3, FloatValue};

#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub a: Point3<FloatValue>,
    pub b: Point3<FloatValue>,
    pub c: Point3<FloatValue>,
    pub normal: Vector3<FloatValue>,
    node_index: usize,
    pub aabb: Aabb<FloatValue, 3>,
}

impl Triangle {
    pub fn new(a: Point3<FloatValue>, b: Point3<FloatValue>, c: Point3<FloatValue>) -> Self {
        let mut aabb = a.aabb();
        aabb.grow_mut(&b);
        aabb.grow_mut(&c);
        Self {
            a,
            b,
            c,
            normal: (b - a).cross(&(c - a)).into(),
            node_index: 0,
            aabb,
        }
    }

    pub fn has_point_in_aabb(&self, aabb: &Aabb<FloatValue, 3>) -> bool {
        aabb.contains(&self.a) || aabb.contains(&self.b) || aabb.contains(&self.c)
    }

    pub fn has_point(&self, vec: Point3<FloatValue>) -> bool {
        relative_eq!(self.a, vec) || relative_eq!(self.b, vec) || relative_eq!(self.c, vec)
    }

    pub fn shares_point_with_triangle(&self, other: Triangle) -> bool {
        self.has_point(other.a) || self.has_point(other.b) || self.has_point(other.c)
    }

    pub fn shares_edge_with_triangle(&self, other: Triangle) -> bool {
        let a = self.has_point(other.a);
        let b = self.has_point(other.b);
        let c = self.has_point(other.c);
        a && b || a && c || b && c
    }

    pub fn intersect(&self, value: FloatValue, axis: usize) -> Option<Line3> {
        let mut intersection = Vec::<Point3<FloatValue>>::with_capacity(3);
        let mut last = &self.c;
        for point in [self.a, self.b, self.c].iter() {
            if relative_eq!(point[axis], value) {
                let mut new_point = *point;
                new_point[axis] = value;
                intersection.push(new_point);
            } else if last[axis] < value && point[axis] > value {
                let ratio = (value - last[axis]) / (point[axis] - last[axis]);
                let mut new_point = last + (point - last) * ratio;
                new_point[axis] = value;
                intersection.push(new_point);
            } else if last[axis] > value && point[axis] < value {
                let ratio = (value - point[axis]) / (last[axis] - point[axis]);
                let mut new_point = point + (last - point) * ratio;
                new_point[axis] = value;
                intersection.push(new_point);
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

    pub fn area(&self) -> FloatValue {
        let ab = self.b - self.a;
        let ac = self.c - self.a;
        0.5 * ab.cross(&ac).norm()
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
