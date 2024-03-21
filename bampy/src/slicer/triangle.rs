use approx::{relative_eq, AbsDiffEq, RelativeEq};
use bvh::{
    aabb::{Aabb, Bounded},
    bounding_hierarchy::BHShape,
};
use nalgebra::{ClosedAdd, ClosedMul, ClosedSub, Point3, Scalar, SimdPartialOrd, Vector3};
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

#[inline(always)]
fn vec_inside_aabb<T: SimdPartialOrd + Scalar + Copy + Float + RelativeEq + approx::AbsDiffEq>(
    vec: &Vector3<T>,
    aabb: &Aabb<T, 3>,
) -> bool {
    macro_rules! within {
        ($axis:ident) => {
            ((vec.$axis >= aabb.min.$axis && vec.$axis <= aabb.max.$axis)
                || relative_eq!(vec.$axis, aabb.min.$axis)
                || relative_eq!(vec.$axis, aabb.max.$axis))
        };
    }
    within!(x) && within!(y) && within!(z)
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
        Self {
            a,
            b,
            c,
            normal: (b - a).cross(&(c - a)).into(),
            node_index: 0,
            aabb: Aabb::with_bounds(
                Point3::new(
                    T::min(T::min(a.x, b.x), c.x),
                    T::min(T::min(a.y, b.y), c.y),
                    T::min(T::min(a.z, b.z), c.z),
                ),
                Point3::new(
                    T::max(T::max(a.x, b.x), c.x),
                    T::max(T::max(a.y, b.y), c.y),
                    T::max(T::max(a.z, b.z), c.z),
                ),
            ),
        }
    }

    pub fn has_point_in_aabb(&self, aabb: &Aabb<T, 3>) -> bool {
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

    pub fn shares_point_with_triangle(&self, other: Triangle<T>) -> bool
    where
        T: RelativeEq + Clone,
        <T as AbsDiffEq>::Epsilon: Clone,
    {
        self.has_vec(other.a) || self.has_vec(other.b) || self.has_vec(other.c)
    }

    pub fn shares_edge_with_triangle(&self, other: Triangle<T>) -> bool
    where
        T: RelativeEq + Clone,
        <T as AbsDiffEq>::Epsilon: Clone,
    {
        let a = self.has_vec(other.a);
        let b = self.has_vec(other.b);
        let c = self.has_vec(other.c);
        a && b || a && c || b && c
    }

    pub fn intersect_z(&self, z: T) -> Option<Line3<T>>
    where
        <T as AbsDiffEq>::Epsilon: Clone,
    {
        let mut intersection = Vec::<Vector3<T>>::with_capacity(3);
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
