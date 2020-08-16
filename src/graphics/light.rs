use specs::prelude::*;

pub struct Light;

impl Component for Light {
    type Storage = VecStorage<Self>;
}

impl Light {}
