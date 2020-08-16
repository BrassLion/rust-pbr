
use super::*;
use specs::prelude::*;

pub struct Pose {
    pub model_matrix: nalgebra::Similarity3<f32>,
}

impl Component for Pose {
    type Storage = VecStorage<Self>;
}
