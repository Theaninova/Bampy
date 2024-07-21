use bvh::aabb::Aabb;
use nalgebra::{Point2, Point3};

use super::{triangle::Triangle, FloatValue};

pub trait ProjectToolpath<T> {
    /// Projects the hypothetical toolpath of the object onto the z plane
    fn project_toolpath_onto_z(&self, z: FloatValue, a: FloatValue) -> Option<T>;
}

pub trait ToolpathIntersects<T>: ProjectToolpath<T> {
    /// Checks if a hypothetical toolpath that draws the object could intersect
    /// with the given point, given the tangent of the angle of the toolhead
    fn toolpath_intersects(&self, point: &Point3<FloatValue>, a: FloatValue) -> bool;
}

pub trait ToolpathIntersection {
    fn toolpath_intersection(&self) {}
}

impl ProjectToolpath<Aabb<FloatValue, 2>> for Aabb<FloatValue, 3> {
    fn project_toolpath_onto_z(&self, z: FloatValue, a: FloatValue) -> Option<Aabb<FloatValue, 2>> {
        if z < self.min.z {
            return None;
        }
        let delta_target_z = z - self.min.z;
        let delta_z = self.max.z - self.min.z;
        let delta = a * (delta_target_z / delta_z);

        let dx = (self.max.x - self.min.x) * delta;
        let dy = (self.max.y - self.min.y) * delta;

        Some(Aabb {
            min: Point2::new(self.min.x - dx, self.min.y - dy),
            max: Point2::new(self.max.x + dx, self.max.y + dy),
        })
    }
}

impl ToolpathIntersects<Aabb<FloatValue, 2>> for Aabb<FloatValue, 3> {
    fn toolpath_intersects(&self, point: &Point3<FloatValue>, a: FloatValue) -> bool {
        if let Some(aabb) = self.project_toolpath_onto_z(point.z, a) {
            aabb.approx_contains_eps(&Point2::new(point.x, point.y), FloatValue::EPSILON)
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub struct Triangle2D {
    pub a: Point2<FloatValue>,
    pub b: Point2<FloatValue>,
    pub c: Point2<FloatValue>,
}

impl ProjectToolpath<Triangle2D> for Triangle {
    fn project_toolpath_onto_z(&self, z: FloatValue, a: FloatValue) -> Option<Triangle2D> {
        if z <= self.aabb.min.z {
            return None;
        }

        let cx = (self.a.x + self.b.x + self.c.x) / 3.0;
        let cy = (self.a.y + self.b.y + self.c.y) / 3.0;

        let expand = z - self.aabb.min.z;

        macro_rules! project {
            ( $val:ident ) => {
                let delta = FloatValue::min(z - self.$val.z, 0.0);
                let dx = self.$val.x - cx;
                let dy = self.$val.y - cy;
                let px = self.$val.x + delta * cx;
                let py = self.$val.y + delta * cy;

                Point2::new(px + expand * dy, py + expand * dx)
            };
        }

        let ap = Point2::new(self.a.x, self.b.y);

        macro_rules! values {
            ( $val:ident ) => {{
                let delta = 2.0 * a * (z - self.$val.z);
                let dx = self.$val.x - cx;
                let dy = self.$val.y - cy;
                Point2::new(self.$val.x + dx * delta, self.$val.y + dy * delta)
            }};
        }

        Some(Triangle2D {
            a: values!(a),
            b: values!(b),
            c: values!(c),
        })
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use bvh::aabb::Aabb;
    use nalgebra::{point, Point3};

    use crate::slicer::{triangle::Triangle, z_projection::ProjectToolpath, FloatValue};

    #[test]
    fn test_project_aabb_toolpath() {
        let aabb = Aabb {
            min: Point3::new(0.0, 0.0, 0.0),
            max: Point3::new(1.0, 1.0, 1.0),
        };
        let a = FloatValue::to_radians(45.0).tan();

        let projected = aabb.project_toolpath_onto_z(0.5, a).unwrap();

        assert_relative_eq!(projected.min.x, -0.5);
        assert_relative_eq!(projected.min.y, -0.5);
        assert_relative_eq!(projected.max.x, 1.5);
        assert_relative_eq!(projected.max.y, 1.5);

        let projected_b = aabb.project_toolpath_onto_z(0.0, a).unwrap();
        assert_relative_eq!(projected_b.min.x, 0.0);
        assert_relative_eq!(projected_b.min.y, 0.0);
        assert_relative_eq!(projected_b.max.x, 1.0);
        assert_relative_eq!(projected_b.max.y, 1.0);
    }

    #[test]
    fn test_project_triangle_toolpath() {
        let triangle = Triangle::new(
            point![0.0, 0.0, 0.0],
            point![0.0, 1.5, 1.0],
            point![-0.6, -1.4, 0.2],
        );
        let a = FloatValue::to_radians(30.0).tan();

        let projected = triangle.project_toolpath_onto_z(0.7, a).unwrap();
        println!("{:?}", projected);

        assert_relative_eq!(projected.a.x, -0.86, epsilon = 0.01);
        assert_relative_eq!(projected.a.y, -0.86, epsilon = 0.01);

        assert_relative_eq!(projected.b.x, -0.54, epsilon = 0.01);
        assert_relative_eq!(projected.b.y, 2.59, epsilon = 0.01);

        assert_relative_eq!(projected.c.x, 2.59, epsilon = 0.01);
        assert_relative_eq!(projected.c.y, 1.94, epsilon = 0.01);
    }
}
