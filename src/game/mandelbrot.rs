use std::convert::Into;
use std::default::Default;
use std::vec::Vec;
use num::Complex;
use num_bigfloat::BigFloat;


// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MandelbrotShaderRepresentation {
    generation: f32,
    zoom: f32,
    center_delta: [f32; 2],
    // a small value corresponding to the maximum error tolerated for the calculation the Mandelbrot set
    epsilon: f32,
    // the number of iterations of the calculation of the Mandelbrot set
    maximum_iterations: u32,
    // the width of the screen
    width: u32,
    // the height of the screen
    height: u32,
    // a value used to calculate the maximum value to consider that the mathematics suite is divergent
    mu: f32,
    must_redraw: u32,
    color_palette_scale: f32,
    _padding: u32,
}

pub struct Mandelbrot {
    pub generation: f32,
    pub zoom: f32,
    pub previous_zoom: f32,
    pub center_delta: [f32; 2],
    // the coordinate of the point in the complex plane in the center of the screen
    // pub center_coordinate: [f32; 2],
    // the coordinate of an orbit point in the complex plane that is in the Mandelbrot set
    // and that is near the coordinate of the point in the complex plane in the center of the screen
    pub near_orbit_coordinate: (BigFloat, BigFloat),
    // a small value corresponding to the maximum error tolerated for the calculation the Mandelbrot set
    pub epsilon: f32,
    // the number of iterations of the calculation of the Mandelbrot set
    maximum_iterations: u32,
    // the width of the screen
    pub width: u32,
    // the height of the screen
    pub height: u32,
    // a value used to calculate the maximum value to consider that the mathematics suite is divergent
    pub mu: f32,
    pub must_redraw: u32,
    pub color_palette_scale: f32,
    pub last_orbit_z: (BigFloat, BigFloat),
    pub last_orbit_iteration: u32,
    pub orbit_point_suite: Vec<[f32; 2]>,
}

// x: -0.81448036, y: 0.18333414,
// x: -0.7955818, y: -0.17171985,
// x: -0.8156346, y: 0.18634154,
// x: -0.80087984, y: 0.1822858
// -0.80087334, 0.18227617
// -0.80266213, 0.18230489
// BigFloat::parse("-8.005649172439378601652614980060010776762e-1").unwrap(),
// BigFloat::parse("1.766690913194066364854892309438271746385e-1").unwrap(),
impl Default for Mandelbrot {
    fn default() -> Self {
        let mut orbit_point_suite = Vec::new();
        orbit_point_suite.resize_with(50000, || [0.0, 0.0]);
        Self {
            generation: 0.0,
            zoom: 100.0,
            previous_zoom: 100.0,
            maximum_iterations: 10000,
            width: 0,
            height: 0,
            mu: 10000.0,
            must_redraw: 0,
            color_palette_scale: 100.0,
            center_delta: [0.0, 0.0],
            // near_orbit_coordinate: [-1.6, 0.0],
            near_orbit_coordinate: (
                BigFloat::parse("-7.475923752064317130670890204311118186069e-1").unwrap(),
                BigFloat::parse("8.440544207773073998562585116696206052408e-2").unwrap(),
            ),
            epsilon: 0.0001,
            last_orbit_z: (0.0.into(),0.0.into()),
            orbit_point_suite,
            last_orbit_iteration: 0,
        }
    }
}

impl Mandelbrot {

    pub fn get_shader_representation(& self) -> MandelbrotShaderRepresentation {
        MandelbrotShaderRepresentation {
            generation: self.generation,
            zoom: self.zoom,
            center_delta: self.center_delta,
            epsilon: self.epsilon,
            maximum_iterations: self.maximum_iterations,
            width: self.width,
            height: self.height,
            mu: self.mu,
            must_redraw: self.must_redraw,
            color_palette_scale: self.color_palette_scale,
            _padding: 0,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn maximum_iterations(&self) -> u32 {
        self.maximum_iterations
    }

    pub fn set_maximum_iterations(&mut self, maximum_iterations: u32) -> &mut Self {
        self.maximum_iterations = maximum_iterations;
        self.calculate_orbit_point_suite(false);
        self
    }

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

    pub fn update(&mut self) {
        self.calculate_orbit_point_suite(true);
    }

    fn calculate_orbit_point_suite(&mut self, partial: bool) {
        // let mut result = vec![];
        // result.resize(self.maximum_iterations as usize, [0.0, 0.0]);
        let two = BigFloat::parse("2.0").unwrap();
        let mu = self.mu.into();
        let c = self.near_orbit_coordinate;
        let mut z: (BigFloat, BigFloat) = self.last_orbit_z;
        let mut i = self.last_orbit_iteration as usize;
        let mut count = 0;
        while i < self.maximum_iterations as usize && (!partial || count < 50) {
            self.orbit_point_suite[i as usize]=[z.0.to_f32(), z.1.to_f32()];
            // z = z * z + c;
            z = (
                z.0 * z.0 - z.1 * z.1 + c.0,
                z.0 * z.1 * two + c.1,
            );
            self.last_orbit_z = z;
            // calculate z.norm
            let z_norm = (z.0 * z.0 + z.1 * z.1);
            if z_norm > mu {
                break;
            }
            i += 1;
            count += 1;
        }
        self.last_orbit_iteration = i as u32;
    }

    pub fn center_to_orbit(&mut self) {
        self.center_delta[0] = 0.0;
        self.center_delta[1] = 0.0;
        self.must_redraw = 0;
    }

    pub fn center_orbit_at(
        &mut self,
        mouse_x: isize,
        mouse_y: isize,
        window_width: u32,
        window_height: u32,
    ) {
        let normalized_mouse_vector = (
            (BigFloat::from_f64(mouse_x as f64) - (BigFloat::from_f64(window_width as f64) / BigFloat::parse("2.0").unwrap())) / (BigFloat::from_f64(window_width as f64)  / BigFloat::parse("2.0").unwrap()),
            (BigFloat::from_f64(mouse_y as f64) - (BigFloat::from_f64(window_height as f64) / BigFloat::parse("2.0").unwrap())) / (BigFloat::from_f64(window_height as f64)  / BigFloat::parse("2.0").unwrap()) * BigFloat::parse("-1.0").unwrap(),
        );
        let delta = (
            normalized_mouse_vector.0 * (BigFloat::from_f64(self.width as f64) / BigFloat::from_f64(self.height as f64)) * BigFloat::from_f64(self.zoom as f64),
            normalized_mouse_vector.1 * BigFloat::from_f64(self.zoom as f64) ,
        );
        self.near_orbit_coordinate.0 += delta.0 + BigFloat::from_f64(self.center_delta[0] as f64);
        self.near_orbit_coordinate.1 += delta.1 + BigFloat::from_f64(self.center_delta[1] as f64);
        self.center_delta[0] = -delta.0.to_f32();
        self.center_delta[1] = -delta.1.to_f32();
        self.last_orbit_iteration = 0;
        self.last_orbit_z = (0.0.into(),0.0.into());
        self.calculate_orbit_point_suite(true);
        self.must_redraw = 0;
        println!(
            "x: {}, y: {}, zoom: {}",
            self.near_orbit_coordinate.0,
            self.near_orbit_coordinate.1,
            self.zoom
        );
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
        self.mu = 10000.0;
        self.must_redraw = 0;
    }

    // implement new for MandelbrotShader, without zoom, x, y, mu
    pub fn new(maximum_iterations: u32, width: u32, height: u32) -> Self {
        let mut value = Self{
            maximum_iterations,
            width,
            height,
            ..Default::default()
        };
        value.calculate_orbit_point_suite(false);
        value
    }
}