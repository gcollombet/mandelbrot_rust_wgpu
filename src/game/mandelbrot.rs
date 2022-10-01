use num::Complex;
use num::complex::ComplexFloat;

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Mandelbrot {
    pub generation: f32,
    pub zoom: f32,
    pub center_delta: [f32; 2],
    // the coordinate of the point in the complex plane in the center of the screen
    // pub center_coordinate: [f32; 2],
    // the coordinate of an orbit point in the complex plane that is in the Mandelbrot set
    // and that is near the coordinate of the point in the complex plane in the center of the screen
    pub near_orbit_coordinate: [f64; 2],
    // a small value corresponding to the maximum error tolerated for the calculation the Mandelbrot set
    pub epsilon: f32,
    // the number of iterations of the calculation of the Mandelbrot set
    pub maximum_iterations: u32,
    // the width of the screen
    pub width: u32,
    // the height of the screen
    pub height: u32,
    // a value used to calculate the maximum value to consider that the mathematics suite is divergent
    pub mu: f32,
    pub must_redraw: u32,
    pub color_palette_scale: f32,
    pub z_square: f32,
}

// x: -0.81448036, y: 0.18333414,
// x: -0.7955818, y: -0.17171985,
// x: -0.8156346, y: 0.18634154,
// x: -0.80087984, y: 0.1822858
// -0.80087334, 0.18227617
// -0.80266213, 0.18230489
impl Default for Mandelbrot {
    fn default() -> Self {
        Self {
            generation: 0.0,
            zoom: 100.0,
            maximum_iterations: 10000,
            width: 0,
            height: 0,
            mu: 10000.0,
            must_redraw: 0,
            color_palette_scale: 100.0,
            center_delta: [0.0, 0.0],
            // near_orbit_coordinate: [-1.6, 0.0],
            near_orbit_coordinate: [-0.8005649172622006, 0.17666909128376448],
            epsilon: 0.0001,
            z_square: 0.0,
        }
    }
}

impl Mandelbrot {

    pub fn center_at(
        &mut self,
        mouse_x: f32,
        mouse_y: f32,
        window_width: u32,
        window_height: u32,
    ) {
        let normalized_mouse_vector = (
            (mouse_x - (window_width as f32 / 2.0)) / (window_width as f32 / 2.0),
            (mouse_y - (window_height as f32 / 2.0)) / (window_height as f32 / 2.0) * -1.0,
        );
        self.center_delta[0] += normalized_mouse_vector.0 * (self.width as f32 / self.height as f32) * self.zoom;
        self.center_delta[1] += normalized_mouse_vector.1 * self.zoom;
        self.must_redraw = 0;
    }

    pub fn calculate_orbit_point_suite(
        &self,
    ) -> Vec<[f32; 2]> {
        let mut result = vec![];
        result.resize(10000, [0.0, 0.0]);
        let mut z = Complex::new(
            0.0_f64,
            0.0_f64,
        );
        let c = Complex::new(
            self.near_orbit_coordinate[0],
            self.near_orbit_coordinate[1],
        );
        let mut i = 0;
        while i < 10000 {
            result[i as usize]=[z.re() as f32, z.im() as f32];
            z = z * z + c;
            if z.norm() > self.mu as f64 {
                break;
            }
            i += 1;
        }
        result
    }

    pub fn center_to_orbit(&mut self) {
        self.center_delta[0] = 0.0;
        self.center_delta[1] = 0.0;
        self.must_redraw = 0;
        // print to console the coordinates
        println!(
            "x: {}, y: {}, zoom: {}",
            self.near_orbit_coordinate[0] as f64 + self.center_delta[0] as f64,
            self.near_orbit_coordinate[1] as f64 + self.center_delta[1] as f64,
            self.zoom
        );
    }

    pub fn center_orbit_at(
        &mut self,
        mouse_x: isize,
        mouse_y: isize,
        window_width: u32,
        window_height: u32,
    ) {
        let normalized_mouse_vector = (
            (mouse_x as f64 - (window_width as f64 / 2.0)) / (window_width as f64 / 2.0),
            (mouse_y as f64 - (window_height as f64 / 2.0)) / (window_height as f64 / 2.0) * -1.0,
        );
        let delta = (
            normalized_mouse_vector.0 * (self.width as f64 / self.height as f64) * self.zoom as f64,
            normalized_mouse_vector.1 * self.zoom as f64,
        );
        self.near_orbit_coordinate[0] += delta.0 + self.center_delta[0] as f64;
        self.near_orbit_coordinate[1] += delta.1 + self.center_delta[1] as f64;
        self.center_delta[0] = -delta.0 as f32;
        self.center_delta[1] = -delta.1 as f32;
        self.must_redraw = 0;
    }

    pub fn zoom_in(&mut self, zoom_factor: f32) {
        self.zoom = (self.zoom as f64 * zoom_factor as f64) as f32;
        self.must_redraw = 0;
    }

    // a function that move the mandelbrot center coordinate by a given vector
    pub fn move_by(&mut self, vector: (f32, f32)) {
        self.center_delta[0] += vector.0;
        self.center_delta[1] += vector.1;
        self.must_redraw = 0;
    }

    pub fn move_by_pixel(&mut self,
                         mouse_x: isize,
                         mouse_y: isize,
                         window_width: u32,
                         window_height: u32,) {
        let normalized_mouse_vector = (
            mouse_x as f32 / (window_width as f32 / 2.0),
            mouse_y as f32 / (window_height as f32 / 2.0) * -1.0,
        );
        self.center_delta[0] -= normalized_mouse_vector.0 * (self.width as f32 / self.height as f32) * self.zoom;
        self.center_delta[1] -= normalized_mouse_vector.1 * self.zoom;
        self.must_redraw = 0;
    }

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
    pub fn zoom_at(&mut self, zoom_factor: f32, mouse_x: f32, mouse_y: f32, window_width: u32, window_height: u32) {
        let normalized_mouse_vector = (
            (mouse_x - (window_width as f32 / 2.0)) / (window_width as f32 / 2.0),
            (mouse_y - (window_height as f32 / 2.0)) / (window_height as f32 / 2.0)
        );
        let scaled_mouse_vector = (
            normalized_mouse_vector.0 * self.zoom,
            normalized_mouse_vector.1 * self.zoom
        );
        self.center_delta[0] += scaled_mouse_vector.0;
        self.center_delta[1] -= scaled_mouse_vector.1;
        let zoomed_scaled_mouse_vector = (
            scaled_mouse_vector.0 * zoom_factor,
            scaled_mouse_vector.1 * zoom_factor
        );
        self.center_delta[0] -= zoomed_scaled_mouse_vector.0;
        self.center_delta[1] += zoomed_scaled_mouse_vector.1;
        self.zoom *= zoom_factor;
        self.must_redraw = 0;
    }

    // function reset the mandelbrot set to its default values
    pub fn reset(&mut self) {
        self.zoom = 100.0;
        self.maximum_iterations = 10000;
        self.mu = 10000.0;
        self.must_redraw = 0;
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