/// https://iquilezles.org/articles/distfunctions/
use nalgebra::{vector, Point, TAffine, Transform, Vector2, Vector3};

use super::FloatValue;

pub trait Sdf<const D: usize> {
    fn sdf(&self, p: Point<FloatValue, D>) -> FloatValue;
}

#[derive(Debug, Clone, Copy)]
pub struct SdfSphere {
    radius: FloatValue,
}

impl SdfSphere {
    pub fn new(radius: FloatValue) -> Self {
        Self { radius }
    }
}

impl Sdf<3> for SdfSphere {
    fn sdf(&self, p: Point<FloatValue, 3>) -> FloatValue {
        p.coords.norm() - self.radius
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SdfBox {
    size: Point<FloatValue, 3>,
}

impl SdfBox {
    pub fn new(size: Point<FloatValue, 3>) -> Self {
        Self { size }
    }
}

impl Sdf<3> for SdfBox {
    fn sdf(&self, p: Point<FloatValue, 3>) -> FloatValue {
        let q = p.coords.abs() - self.size.coords;
        q.sup(&Vector3::zeros()).add_scalar(q.max().min(0.0)).norm()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SdfInfiniteCone {
    angle: Vector2<FloatValue>,
}

impl SdfInfiniteCone {
    pub fn new(angle: FloatValue) -> Self {
        Self {
            angle: vector![angle.sin(), angle.cos()],
        }
    }
}

impl Sdf<3> for SdfInfiniteCone {
    fn sdf(&self, p: Point<FloatValue, 3>) -> FloatValue {
        let q = vector![p.coords.xy().norm(), p.z];
        let d = (q - self.angle.scale(q.dot(&self.angle).max(0.0))).norm();
        if q.x * self.angle.y - q.y * self.angle.x > 0.0 {
            d
        } else {
            -d
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SdfTransform<T: Sdf<3>> {
    sdf: T,
    transform: Transform<FloatValue, TAffine, 3>,
}

impl<T: Sdf<3>> SdfTransform<T> {
    fn new(sdf: T, transform: Transform<FloatValue, TAffine, 3>) -> Self {
        Self { sdf, transform }
    }
}

impl<T: Sdf<3>> Sdf<3> for SdfTransform<T> {
    fn sdf(&self, p: Point<FloatValue, 3>) -> FloatValue {
        self.sdf.sdf(self.transform.inverse_transform_point(&p))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SdfScale<const D: usize, T: Sdf<D>> {
    sdf: T,
    scale: FloatValue,
}

impl<const D: usize, T: Sdf<D>> SdfScale<D, T> {
    fn new(sdf: T, scale: FloatValue) -> Self {
        Self { sdf, scale }
    }
}

impl<const D: usize, T: Sdf<D>> Sdf<D> for SdfScale<D, T> {
    fn sdf(&self, p: Point<FloatValue, D>) -> FloatValue {
        self.sdf.sdf(p / self.scale) * self.scale
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SdfOnion<T: Sdf<3>> {
    sdf: T,
    thickness: FloatValue,
}

impl<T: Sdf<3>> SdfOnion<T> {
    fn new(sdf: T, thickness: FloatValue) -> Self {
        Self { sdf, thickness }
    }
}

impl<T: Sdf<3>> Sdf<3> for SdfOnion<T> {
    fn sdf(&self, p: Point<FloatValue, 3>) -> FloatValue {
        self.sdf.sdf(p).abs() - self.thickness
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SdfUnion<const D: usize, T: Sdf<D>, U: Sdf<D>> {
    sdf_a: T,
    sdf_b: U,
}

impl<const D: usize, T: Sdf<D>, U: Sdf<D>> SdfUnion<D, T, U> {
    fn new(sdf_a: T, sdf_b: U) -> Self {
        Self { sdf_a, sdf_b }
    }
}

impl<const D: usize, T: Sdf<D>, U: Sdf<D>> Sdf<D> for SdfUnion<D, T, U> {
    fn sdf(&self, p: Point<FloatValue, D>) -> FloatValue {
        self.sdf_a.sdf(p).min(self.sdf_b.sdf(p))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SdfIntersection<const D: usize, T: Sdf<D>, U: Sdf<D>> {
    sdf_a: T,
    sdf_b: U,
}

impl<const D: usize, T: Sdf<D>, U: Sdf<D>> SdfIntersection<D, T, U> {
    fn new(sdf_a: T, sdf_b: U) -> Self {
        Self { sdf_a, sdf_b }
    }
}

impl<const D: usize, T: Sdf<D>, U: Sdf<D>> Sdf<D> for SdfIntersection<D, T, U> {
    fn sdf(&self, p: Point<FloatValue, D>) -> FloatValue {
        self.sdf_a.sdf(p).max(self.sdf_b.sdf(p))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SdfDifference<const D: usize, T: Sdf<D>, U: Sdf<D>> {
    sdf_a: T,
    sdf_b: U,
}

impl<const D: usize, T: Sdf<D>, U: Sdf<D>> SdfDifference<D, T, U> {
    fn new(sdf_a: T, sdf_b: U) -> Self {
        Self { sdf_a, sdf_b }
    }
}

impl<const D: usize, T: Sdf<D>, U: Sdf<D>> Sdf<D> for SdfDifference<D, T, U> {
    fn sdf(&self, p: Point<FloatValue, D>) -> FloatValue {
        self.sdf_a.sdf(p).max(-self.sdf_b.sdf(p))
    }
}

pub trait SdfOperators<const D: usize>: Sdf<D> {
    fn union(self, other: Self) -> SdfUnion<D, Self, Self>
    where
        Self: Sized,
    {
        SdfUnion::new(self, other)
    }

    fn intersection(self, other: Self) -> SdfIntersection<D, Self, Self>
    where
        Self: Sized,
    {
        SdfIntersection::new(self, other)
    }

    fn difference(self, other: Self) -> SdfDifference<D, Self, Self>
    where
        Self: Sized,
    {
        SdfDifference::new(self, other)
    }
}

impl<const D: usize, T: Sdf<D>> SdfOperators<D> for T {}

pub trait Sdf3dModifiers: Sdf<3> {
    /// exact
    fn transform(self, transform: Transform<FloatValue, TAffine, 3>) -> SdfTransform<Self>
    where
        Self: Sized,
    {
        SdfTransform::new(self, transform)
    }

    /// exact
    fn translate(self, translation: Point<FloatValue, 3>) -> SdfTransform<Self>
    where
        Self: Sized,
    {
        self.transform(Transform::from_matrix_unchecked(
            nalgebra::Matrix4::new_translation(&translation.coords),
        ))
    }

    /// exact
    fn rotate(self, rotation: nalgebra::UnitQuaternion<FloatValue>) -> SdfTransform<Self>
    where
        Self: Sized,
    {
        self.transform(Transform::from_matrix_unchecked(rotation.to_homogeneous()))
    }

    /// exact
    fn scale(self, scale: FloatValue) -> SdfScale<3, Self>
    where
        Self: Sized,
    {
        SdfScale::new(self, scale)
    }

    /// exact
    ///
    /// For carving interiors or giving thickness to primitives,
    /// without performing expensive boolean operations
    /// and without distorting the distance field into a bound,
    /// one can use "onioning".
    /// You can use it multiple times to create concentric layers in your SDF.
    fn onion(self, thickness: FloatValue) -> SdfOnion<Self>
    where
        Self: Sized,
    {
        SdfOnion::new(self, thickness)
    }
}

impl<T: Sdf<3>> Sdf3dModifiers for T {}
