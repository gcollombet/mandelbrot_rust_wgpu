// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Mandelbrot {
    pub seed: f32,
    pub zoom: f32,
    pub x: f32,
    pub y: f32,
    pub maximum_iterations: u32,
    pub  width: u32,
    pub height: u32,
    pub mu: f32,
    pub is_rendered: u32,
}

impl Default for Mandelbrot {
    fn default() -> Self {
        Self {
            seed: 0.0,
            zoom: 100.0,
            x: 0.0,
            y: 0.0,
            maximum_iterations: 10000,
            width: 0,
            height: 0,
            mu: 10000.0,
            is_rendered: 0,
        }
    }
}

impl Mandelbrot {
    // a function that zoom in the mandelbrot set by a given factor.
    // the function take as parameters :
    // - the zoom factor
    // - the x and y coordinates of the mouse
    // - the width and height of the window
    // The function compute the normalized vector of the mouse position relatively to the window center in a variable called "normalized_mouse_vector"
    // Then it multiply the mouse_vector by the zoom factor in a new variable called "scaled_mouse_vector"
    // Then it add the "scaled_mouse_vector" to the current x and y coordinates of the mandelbrot set
    // Then it multiply the "scaled_mouse_vector" by the zoom factor in a new variable called "zoomed_scaled_mouse_vector"
    // Then it add the "zoomed_scaled_mouse_vector" times -1 to the current x and y coordinates of the mandelbrot set
    pub fn zoom_in(&mut self, zoom_factor: f32, mouse_x: f32, mouse_y: f32, window_width: u32, window_height: u32) {
        let normalized_mouse_vector = (
            (mouse_x - (window_width as f32 / 2.0)) / (window_width as f32 / 2.0),
            (mouse_y - (window_height as f32 / 2.0)) / (window_height as f32 / 2.0)
        );
        let scaled_mouse_vector = (
            normalized_mouse_vector.0 * self.zoom,
            normalized_mouse_vector.1 * self.zoom
        );
        self.x += scaled_mouse_vector.0;
        self.y -= scaled_mouse_vector.1;
        let zoomed_scaled_mouse_vector = (
            scaled_mouse_vector.0 * zoom_factor,
            scaled_mouse_vector.1 * zoom_factor
        );
        self.x -= zoomed_scaled_mouse_vector.0;
        self.y += zoomed_scaled_mouse_vector.1;
        self.zoom *= zoom_factor;
        self.is_rendered = 0;
    }

    // function reset the mandelbrot set to its default values
    pub fn reset(&mut self) {
        self.zoom = 100.0;
        self.maximum_iterations = 10000;
        self.mu = 10000.0;
        self.is_rendered = 0;
    }

    // implement new for MandelbrotShader, without zoom, x, y, mu
    pub fn new(maximum_iterations: u32, width: u32, height: u32) -> Self {
        Self {
            maximum_iterations,
            width,
            height,
            ..Default::default()
        }
    }
}