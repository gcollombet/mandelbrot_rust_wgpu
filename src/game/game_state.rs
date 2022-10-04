use winit::event::Event;
use crate::game::Game;

pub trait GameState {
    fn update(&mut self, game: &mut Game);
    fn input(&mut self, event: &Event<()>, game: &mut Game);
}