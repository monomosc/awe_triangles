use bevy::{prelude::{Resource, ResMut, Res, Component}, reflect::Reflect};

#[derive(Debug, Resource, Reflect)]
pub struct Speed(pub f32);

impl Default for Speed {
    fn default() -> Self {
        Self(0.1)
    }
}

#[derive(Debug, Resource, Reflect, Default)]
pub struct Paused(pub bool);

pub fn is_not_paused(paused: Res<Paused>) -> bool {
    return paused.0 == false;
}



#[derive(Debug, Component, Reflect)]
pub struct VelocityVector;