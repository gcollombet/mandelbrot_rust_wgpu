use std::rc::Rc;

use winit::event::{
    ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Fullscreen, Icon, WindowBuilder};

// import game module
use crate::game::Game;

pub async fn run() {
    // print control
    print_controls();
    // create event loop
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title("Realtime Mandelbrot Explorer");
    window.set_inner_size(winit::dpi::LogicalSize::new(800.0, 800.0));
    // decode a png file into a vector of u8
    let icon = image::load_from_memory(include_bytes!("../assets/logo.png"))
        .unwrap()
        .to_rgba8();
    // add an icon to the window
    window.set_window_icon(Some(Icon::from_rgba(icon.into_raw(), 256, 256).unwrap()));
    let window = Rc::new(window);
    // create a reference counted pointer to the window
    let mut game = Game::new(window.clone()).await;
    event_loop.run(move |event, _, control_flow| game.input(event, control_flow));
}

// a function that print the keys to control the game
fn print_controls() {
    println!("Controls:");
    println!("  - Mouse wheel to zoom at center of screen");
    println!("  - Left mouse pressed to move");
    println!("  - Right mouse pressed to rotate");
    println!("  - Arrow keys or ZQSD to move");
    println!("  - A and E to rotate left and right");
    println!("  - Numpad + and - to change the zoom speed");
    println!("  - Numpad / and * to change the iteration count");
    println!("  - Space pause the animation");
    println!("  - Entrer to reset the zoom and rotation");
    println!("  - Page up/down to increase/decrease the color palette scale");
    println!("  - F11 to toggle fullscreen");
    println!("  - Escape to quit");
}
