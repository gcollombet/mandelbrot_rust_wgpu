use bytemuck::{Pod, Zeroable};
use winit::event::Event;
use crate::game::engine::Engine;
use crate::game::Game;

pub trait GameState {
    fn update(&mut self, engine: &mut Engine, delta_time: f32);
    fn input(&mut self, event: &Event<()>, engine: &mut Engine);
}