use bvh::{aabb::Aabb, bvh::Bvh};
use nalgebra::{ClosedAdd, ClosedMul, ClosedSub, Scalar, SimdPartialOrd};
use num::{Float, FromPrimitive};

use super::triangle::Triangle;

#[derive(Debug)]
pub struct Mesh<T: SimdPartialOrd + Scalar + Copy> {
    pub aabb: Aabb<T, 3>,
    pub bvh: Bvh<T, 3>,
    pub triangles: Vec<Triangle<T>>,
}

impl<T> From<Vec<Triangle<T>>> for Mesh<T>
where
    T: SimdPartialOrd + Scalar + Copy + ClosedMul + ClosedAdd + ClosedSub + Float + FromPrimitive,
{
    fn from(mut triangles: Vec<Triangle<T>>) -> Self {
        Self {
            aabb: triangles
                .get(0)
                .map(|triangle| {
                    let mut aabb = triangle.aabb;
                    for triangle in triangles.iter().skip(1) {
                        aabb.join_mut(&triangle.aabb);
                    }
                    aabb
                })
                .unwrap_or_else(|| Aabb::empty()),
            bvh: Bvh::build(&mut triangles),
            triangles,
        }
    }
}
