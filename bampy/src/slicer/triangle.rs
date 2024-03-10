use approx::{relative_eq, AbsDiffEq, RelativeEq};
use bvh::{
    aabb::{Aabb, Bounded},
    bounding_hierarchy::BHShape,
};
use nalgebra::{ClosedAdd, ClosedMul, ClosedSub, Scalar, SimdPartialOrd, Vector3};
use num::{Float, FromPrimitive};

use super::line::Line3;

#[derive(Debug, Clone, Copy)]
pub struct Triangle<T: SimdPartialOrd + Scalar + Copy> {
    pub a: Vector3<T>,
    pub b: Vector3<T>,
    pub c: Vector3<T>,
    pub normal: Vector3<T>,
    node_index: usize,
    pub aabb: Aabb<T, 3>,
}

fn vec_inside_aabb<T: SimdPartialOrd + Scalar + Copy + Float>(
    vec: &Vector3<T>,
    aabb: &Aabb<T, 3>,
) -> bool {
    vec.x >= aabb.min.x
        && vec.y >= aabb.min.y
        && vec.z >= aabb.min.z
        && vec.x <= aabb.max.x
        && vec.y <= aabb.max.y
        && vec.z <= aabb.max.z
}

impl<T> Triangle<T>
where
    T: SimdPartialOrd
        + RelativeEq
        + Scalar
        + Copy
        + ClosedMul
        + ClosedAdd
        + ClosedSub
        + Float
        + FromPrimitive,
{
    pub fn new(a: Vector3<T>, b: Vector3<T>, c: Vector3<T>) -> Self {
        let normal = (b - a).cross(&(c - a));
        let mut aabb = Aabb::with_bounds(a.into(), b.into());
        aabb.grow_mut(&c.into());
        Self {
            a,
            b,
            c,
            normal: normal.into(),
            node_index: 0,
            aabb,
        }
    }

    pub fn intersects_aabb(&self, aabb: &Aabb<T, 3>) -> bool {
        vec_inside_aabb(&self.a, aabb)
            || vec_inside_aabb(&self.b, aabb)
            || vec_inside_aabb(&self.c, aabb)
    }

    pub fn has_vec(&self, vec: Vector3<T>) -> bool
    where
        T: RelativeEq + Clone,
        <T as AbsDiffEq>::Epsilon: Clone,
    {
        relative_eq!(self.a, vec) || relative_eq!(self.b, vec) || relative_eq!(self.c, vec)
    }

    pub fn connected_with_triangle(&self, other: Triangle<T>) -> bool
    where
        T: RelativeEq + Clone,
        <T as AbsDiffEq>::Epsilon: Clone,
    {
        self.has_vec(other.a) || self.has_vec(other.b) || self.has_vec(other.c)
    }

    pub fn intersect_z(&self, z: T) -> Option<Line3<T>> {
        let mut intersection = Vec::with_capacity(3);
        let mut last = self.c;
        for point in [self.a, self.b, self.c].iter() {
            if relative_eq!(point.z, z) {
                intersection.push(*point);
            } else if last.z < z && point.z > z || last.z > z && point.z < z {
                intersection.push(last.lerp(&point, (z - last.z) / (point.z - last.z)));
            }
            last = *point;
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

impl<T: SimdPartialOrd + Scalar + Copy> Bounded<T, 3> for Triangle<T> {
    fn aabb(&self) -> Aabb<T, 3> {
        self.aabb
    }
}

impl<T: SimdPartialOrd + Scalar + Copy> BHShape<T, 3> for Triangle<T> {
    fn set_bh_node_index(&mut self, node_index: usize) {
        self.node_index = node_index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}
