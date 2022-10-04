extern crate core;

mod game;
mod runner;

fn main() {
    pollster::block_on(runner::run());
}
