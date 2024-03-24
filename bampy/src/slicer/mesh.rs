use super::{triangle::Triangle, FloatValue};
use bvh::{aabb::Aabb, bvh::Bvh};

#[derive(Debug)]
pub struct Mesh {
    pub aabb: Aabb<FloatValue, 3>,
    pub bvh: Bvh<FloatValue, 3>,
    pub triangles: Vec<Triangle>,
}

impl From<Vec<Triangle>> for Mesh {
    fn from(mut triangles: Vec<Triangle>) -> Self {
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
