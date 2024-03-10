use std::fmt::Debug;

pub mod base_slices;
pub mod line;
pub mod mesh;
pub mod split_surface;
pub mod triangle;

#[derive(Debug)]
pub struct SlicerOptions {
    pub layer_height: f32,
}
