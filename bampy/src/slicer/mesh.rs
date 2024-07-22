use super::{
    axis::Axis,
    base_slices::BaseSlice,
    line::Line3,
    slice_path::{SlicePath, SurfacePathIterator},
    triangle::Triangle,
    FloatValue,
};
use bvh::{
    aabb::Aabb,
    bvh::{Bvh, BvhNode},
};

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

impl Mesh {
    pub fn slice_paths<'a>(
        self: &'a Mesh,
        axis: Axis,
        slice_height: FloatValue,
    ) -> impl Iterator<Item = Vec<SlicePath>> + 'a {
        self.slice_base_slices(axis, slice_height)
            .map(|slice| slice.find_paths())
            .filter(|paths| !paths.is_empty())
    }

    pub fn slice_surface(&self, axis: Axis, nozzle_width: FloatValue) -> SurfacePathIterator {
        SurfacePathIterator::new(self, axis, nozzle_width)
    }

    pub fn slice_base_slices<'a>(
        self: &'a Mesh,
        axis: Axis,
        slice_height: FloatValue,
    ) -> impl Iterator<Item = BaseSlice> + 'a {
        let layer_count = ((self.aabb.max[axis as usize] - self.aabb.min[axis as usize])
            / slice_height)
            .floor() as usize;

        (0..=layer_count).map(move |i| {
            let layer = i as FloatValue * slice_height + self.aabb.min[axis as usize];
            let mut base_slice = BaseSlice {
                i,
                d: layer,
                axis,
                lines: vec![],
            };

            let mut stack = Vec::<usize>::with_capacity(self.bvh.nodes.len());
            stack.push(0);
            while let Some(i) = stack.pop() {
                match self.bvh.nodes[i] {
                    BvhNode::Node {
                        parent_index: _,
                        child_l_index,
                        child_l_aabb,
                        child_r_index,
                        child_r_aabb,
                    } => {
                        assert!(child_l_aabb.min[axis as usize] <= child_l_aabb.max[axis as usize]);
                        assert!(child_r_aabb.min[axis as usize] <= child_r_aabb.max[axis as usize]);
                        if layer >= child_l_aabb.min[axis as usize]
                            && layer <= child_l_aabb.max[axis as usize]
                        {
                            stack.push(child_l_index);
                        }
                        if layer >= child_r_aabb.min[axis as usize]
                            && layer <= child_r_aabb.max[axis as usize]
                        {
                            stack.push(child_r_index);
                        }
                    }
                    BvhNode::Leaf {
                        parent_index: _,
                        shape_index,
                    } => {
                        for line in self.triangles[shape_index].intersect(layer, axis as usize) {
                            base_slice.lines.push(line);
                        }
                    }
                }
            }

            base_slice
        })
    }

    pub fn outline_base_slice(&self, axis: Axis) -> BaseSlice {
        let mut base_slice = BaseSlice {
            i: 0,
            d: 0.0,
            axis,
            lines: vec![],
        };
        'triangle: for (triangle_index, triangle) in self.triangles.iter().enumerate() {
            let mut stack = Vec::<usize>::with_capacity(self.bvh.nodes.len());
            stack.push(0);
            let mut ab = false;
            let mut ac = false;
            let mut bc = false;
            while let Some(i) = stack.pop() {
                match self.bvh.nodes[i] {
                    BvhNode::Node {
                        parent_index: _,
                        child_l_index,
                        child_l_aabb,
                        child_r_index,
                        child_r_aabb,
                    } => {
                        let coords = [triangle.a, triangle.b, triangle.c];
                        macro_rules! match_aabb {
                            ($side:expr) => {
                                coords
                                    .iter()
                                    .map(|point| {
                                        $side.approx_contains_eps(point, FloatValue::EPSILON) as i32
                                    })
                                    .sum::<i32>()
                                    >= 2
                            };
                        }
                        if match_aabb!(child_l_aabb) {
                            stack.push(child_l_index);
                        }
                        if match_aabb!(child_r_aabb) {
                            stack.push(child_r_index);
                        }
                    }
                    BvhNode::Leaf {
                        parent_index: _,
                        shape_index,
                    } => {
                        if triangle_index == shape_index {
                            continue;
                        }
                        let other = &self.triangles[shape_index];
                        let a = other.has_point(triangle.a);
                        let b = other.has_point(triangle.b);
                        let c = other.has_point(triangle.c);
                        ab = ab || (a && b);
                        ac = ac || (a && c);
                        bc = bc || (b && c);

                        if ab && ac && bc {
                            continue 'triangle;
                        }
                    }
                }
            }
            if !ab {
                base_slice.lines.push(Line3::new(triangle.a, triangle.b));
            }
            if !ac {
                base_slice.lines.push(Line3::new(triangle.a, triangle.c));
            }
            if !bc {
                base_slice.lines.push(Line3::new(triangle.b, triangle.c));
            }
        }
        base_slice
    }
}
