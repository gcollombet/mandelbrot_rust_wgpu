use std::cell::RefCell;
use std::convert::Into;
use std::default::Default;
use std::ops::Deref;
use std::rc::Rc;
use std::vec::Vec;

use bytemuck::{Pod, Zeroable};
use num_bigfloat::BigFloat;

use to_buffer_representation_derive::ToBufferRepresentation;

use crate::game::to_buffer_representation::ToBufferRepresentation;

// use array

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Copy, Clone, Pod, Zeroable, ToBufferRepresentation)]
pub struct MandelbrotData {
    pub generation: u32,
    pub time_elapsed: f32,
    pub zoom: f32,
    pub angle: f32,
    pub center_delta: [f32; 2],
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
    pub color_palette_scale: f32,
}

impl MandelbrotData {
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn from(&mut self, other: &MandelbrotData) {
        self.generation = other.generation;
        self.time_elapsed = other.time_elapsed;
        self.zoom = other.zoom;
        self.center_delta[0] = other.center_delta[0];
        self.center_delta[1] = other.center_delta[1];
        self.epsilon = other.epsilon;
        self.maximum_iterations = other.maximum_iterations;
        self.width = other.width;
        self.height = other.height;
        self.mu = other.mu;
        self.color_palette_scale = other.color_palette_scale;
        self.angle = other.angle;
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn center_at(&mut self, mouse_x: f32, mouse_y: f32, window_width: u32, window_height: u32) {
        let normalized_mouse_vector = (
            (mouse_x - (window_width as f32 / 2.0)) / (window_width as f32 / 2.0),
            (mouse_y - (window_height as f32 / 2.0)) / (window_height as f32 / 2.0) * -1.0,
        );
        self.center_delta[0] +=
            normalized_mouse_vector.0 * (self.width as f32 / self.height as f32) * self.zoom;
        self.center_delta[1] += normalized_mouse_vector.1 * self.zoom;
    }

    pub fn center_to_orbit(&mut self) {
        self.center_delta[0] = 0.0;
        self.center_delta[1] = 0.0;
    }

    pub fn zoom_in(&mut self, zoom_factor: f32) {
        self.zoom = (self.zoom as f64 * zoom_factor as f64) as f32;
    }

    // a function that move the mandelbrot center coordinate by a given vector
    pub fn move_by(&mut self, vector: (f32, f32)) {
        if vector.0 != 0.0 || vector.1 != 0.0 {
            // rotate the vector by the angle of the mandelbrot
            let vector = (
                vector.0 * self.angle.cos() - vector.1 * self.angle.sin(),
                vector.0 * self.angle.sin() + vector.1 * self.angle.cos(),
            );
            self.center_delta[0] += vector.0 * self.zoom.min(1.0);
            self.center_delta[1] += vector.1 * self.zoom.min(1.0);
        }
    }

    pub fn move_by_pixel(
        &mut self,
        mouse_x: isize,
        mouse_y: isize,
        window_width: u32,
        window_height: u32,
    ) {
        let normalized_mouse_vector = (
            mouse_x as f32 / (window_width as f32 / 2.0),
            mouse_y as f32 / (window_height as f32 / 2.0) * -1.0,
        );
        // rotate the vector by the angle of the mandelbrot
        let normalized_mouse_vector = (
            normalized_mouse_vector.0 * self.angle.cos()
                - normalized_mouse_vector.1 * self.angle.sin(),
            normalized_mouse_vector.0 * self.angle.sin()
                + normalized_mouse_vector.1 * self.angle.cos(),
        );
        self.center_delta[0] -=
            normalized_mouse_vector.0 * (self.width as f32 / self.height as f32) * self.zoom;
        self.center_delta[1] -= normalized_mouse_vector.1 * self.zoom;
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
    pub fn zoom_at(
        &mut self,
        zoom_factor: f32,
        mouse_x: f32,
        mouse_y: f32,
        window_width: u32,
        window_height: u32,
    ) {
        let normalized_mouse_vector = (
            (mouse_x - (window_width as f32 / 2.0)) / (window_width as f32 / 2.0),
            (mouse_y - (window_height as f32 / 2.0)) / (window_height as f32 / 2.0),
        );
        let scaled_mouse_vector = (
            normalized_mouse_vector.0 * self.zoom,
            normalized_mouse_vector.1 * self.zoom,
        );
        self.center_delta[0] += scaled_mouse_vector.0;
        self.center_delta[1] -= scaled_mouse_vector.1;
        let zoomed_scaled_mouse_vector = (
            scaled_mouse_vector.0 * zoom_factor,
            scaled_mouse_vector.1 * zoom_factor,
        );
        self.center_delta[0] -= zoomed_scaled_mouse_vector.0;
        self.center_delta[1] += zoomed_scaled_mouse_vector.1;
        self.zoom *= zoom_factor;
    }

    // function reset the mandelbrot set to its default values
    pub fn reset(&mut self) {
        self.zoom = 3.0;
        self.angle = 0.0;
    }
}

pub struct MandelbrotEngine {
    pub near_orbit_coordinate: (BigFloat, BigFloat),
    pub last_orbit_z: (BigFloat, BigFloat),
    pub last_orbit_iteration: u32,
    pub orbit_point_suite: Rc<RefCell<Vec<[f32; 2]>>>,
    pub data: Rc<RefCell<MandelbrotData>>,
}

// x: -0.81448036, y: 0.18333414,
// x: -0.7955818, y: -0.17171985,
// x: -0.8156346, y: 0.18634154,
// x: -0.80087984, y: 0.1822858
// -0.80087334, 0.18227617
// -0.80266213, 0.18230489
// BigFloat::parse("-8.005649172439378601652614980060010776762e-1").unwrap(),
// BigFloat::parse("1.766690913194066364854892309438271746385e-1").unwrap(),
// x: -7.500089440803277955738568318416497381310e-1, y: 2.404459737086860128597851267463334330760e-3, zoom: 0.00000000000000000000000000000000087084745
// x: -7.499996610927331414359149762839498275786e-1, y: 2.114906405075616737857913420725088627271e-3, zoom: 0.000000000000000000000000000000000008015796
// x: -7.627677549887342372397302854143029545734e-1, y: 8.931480905527971786649375475419219722965e-2, zoom: 0.0000000000000000004938197
// BigFloat::parse("-7.475923752064317130670890204311118186069e-1").unwrap(),
// BigFloat::parse("8.440544207773073998562585116696206052408e-2").unwrap(),
// x: 2.500049188204330247430641668375026514745e-1, y: -1.600272803318229356053074373282130870020e-8, zoom: 0.0000000000000000000000000000000002284585
//x: -1.749978147146353430779101439642200950582, y: 8.341926669432979672418164970092702884110e-12, zoom: 0.000000004474571
//x: -1.749922480927599928271333687542289453030433024473703345006508521395924860650654081299355473751219976598678491114359225427863893386542382475600444642781285056640754, y: -0.000000000000959502198314327569948975707202650233401883670299418141500240641361234506320676962536124684582340235944852850785763764700482870569928474715774446003497, zoom: 0.000000004474571
// BigFloat::parse("-8.005649172439378601652614980060010776762e-1").unwrap(),
// BigFloat::parse("1.766690913194066364854892309438271746385e-1").unwrap(),
// BigFloat::parse("-1.749922480927599928271333687542289453030433024473703345006508521395924860650654081299355473751219976598678491114359225427863893386542382475600444642781285056640754").unwrap(),
// BigFloat::parse("-0.000000000000959502198314327569948975707202650233401883670299418141500240641361234506320676962536124684582340235944852850785763764700482870569928474715774446003497").unwrap(),
// x: -5.572506229492064091994520833394481793049e-1, y: 6.355989165839159099969652617613951003226e-1, zoom: 0.0000000000000000000000000000000000015172783
impl Default for MandelbrotEngine {
    fn default() -> Self {
        let mut orbit_point_suite = Vec::new();
        orbit_point_suite.resize_with(1000000, || [0.0, 0.0]);
        Self {
            near_orbit_coordinate: (
                BigFloat::parse("-1.749922480927599928271333687542289453030433024473703345006508521395924860650654081299355473751219976598678491114359225427863893386542382475600444642781285056640754").unwrap(),
                BigFloat::parse("-0.000000000000959502198314327569948975707202650233401883670299418141500240641361234506320676962536124684582340235944852850785763764700482870569928474715774446003497").unwrap(),
            ),
            last_orbit_z: (0.0.into(), 0.0.into()),
            orbit_point_suite: Rc::new(RefCell::new(orbit_point_suite)),
            last_orbit_iteration: 0,
            data: Rc::new(RefCell::new(MandelbrotData {
                generation: 0,
                time_elapsed: 0.0,
                zoom: 3.0,
                center_delta: [0.0, 0.0],
                epsilon: 0.0001,
                maximum_iterations: 100,
                width: 0,
                height: 0,
                mu: 10000.0,
                color_palette_scale: 100.0,
                angle: 0.0,
            })),
        }
    }
}

impl MandelbrotEngine {
    pub fn resize(&mut self, width: u32, height: u32) {
        self.data.deref().borrow_mut().resize(width, height);
    }

    pub fn maximum_iterations(&self) -> u32 {
        self.data.borrow().maximum_iterations
    }

    pub fn set_maximum_iterations(&mut self, maximum_iterations: u32) -> &mut Self {
        self.data.deref().borrow_mut().maximum_iterations = maximum_iterations;
        self.calculate_orbit_point_suite(false);
        self
    }

    pub fn zoom(&self) -> f32 {
        self.data.borrow().zoom
    }

    pub fn set_zoom(&mut self, zoom: f32) -> &mut Self {
        self.data.deref().borrow_mut().zoom = zoom;
        self
    }

    pub fn update(&mut self, delta_time: f32) {
        self.data.deref().borrow_mut().generation += 1;
        self.data.deref().borrow_mut().time_elapsed += delta_time;
        // if the center is too far away from the orbit, reset the orbit
        let delta = self.data.deref().borrow().center_delta;
        // calculate the delta length
        let delta_length = delta[0].abs() + delta[1].abs();
        if delta_length >= self.zoom() * 2.0 {
            self.near_orbit_coordinate.0 += BigFloat::from_f32(delta[0]);
            self.near_orbit_coordinate.1 += BigFloat::from_f32(delta[1]);
            self.data.deref().borrow_mut().center_delta = [0.0, 0.0];
            self.last_orbit_iteration = 0;
            self.last_orbit_z = (0.0.into(), 0.0.into());
            self.calculate_orbit_point_suite(false);
        } else {
            self.calculate_orbit_point_suite(true);
        }
    }

    fn calculate_orbit_point_suite(&mut self, partial: bool) {
        let two = BigFloat::parse("2.0").unwrap();
        let mu = self.data.borrow().mu.into();
        let c = self.near_orbit_coordinate;
        let mut z: (BigFloat, BigFloat) = self.last_orbit_z;
        let mut i = self.last_orbit_iteration as usize;
        let mut count = 0;
        while i < self.data.borrow().maximum_iterations as usize && (!partial || count < 50) {
            self.orbit_point_suite.deref().borrow_mut()[i as usize] = [z.0.to_f32(), z.1.to_f32()];
            // z = z * z + c;
            z = (z.0 * z.0 - z.1 * z.1 + c.0, z.0 * z.1 * two + c.1);
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

    pub fn center_orbit_at(
        &mut self,
        mouse_x: isize,
        mouse_y: isize,
        window_width: u32,
        window_height: u32,
    ) {
        let normalized_mouse_vector = (
            (BigFloat::from_f64(mouse_x as f64)
                - (BigFloat::from_f64(window_width as f64) / BigFloat::parse("2.0").unwrap()))
                / (BigFloat::from_f64(window_width as f64) / BigFloat::parse("2.0").unwrap()),
            (BigFloat::from_f64(mouse_y as f64)
                - (BigFloat::from_f64(window_height as f64) / BigFloat::parse("2.0").unwrap()))
                / (BigFloat::from_f64(window_height as f64) / BigFloat::parse("2.0").unwrap())
                * BigFloat::parse("-1.0").unwrap(),
        );
        let delta = (
            normalized_mouse_vector.0
                * (BigFloat::from_f64(self.data.borrow().width as f64)
                    / BigFloat::from_f64(self.data.borrow().height as f64))
                * BigFloat::from_f64(self.data.borrow().zoom as f64),
            normalized_mouse_vector.1 * BigFloat::from_f64(self.data.borrow().zoom as f64),
        );
        self.near_orbit_coordinate.0 +=
            delta.0 + BigFloat::from_f64(self.data.borrow().center_delta[0] as f64);
        self.near_orbit_coordinate.1 +=
            delta.1 + BigFloat::from_f64(self.data.borrow().center_delta[1] as f64);
        self.data.deref().borrow_mut().center_delta[0] = -delta.0.to_f32();
        self.data.deref().borrow_mut().center_delta[1] = -delta.1.to_f32();
        self.last_orbit_iteration = 0;
        self.last_orbit_z = (0.0.into(), 0.0.into());
        self.calculate_orbit_point_suite(true);
    }

    // implement new for MandelbrotShader, without zoom, x, y, mu
    pub fn new(maximum_iterations: u32, width: u32, height: u32) -> Self {
        let mut value = Self {
            ..Default::default()
        };
        value.calculate_orbit_point_suite(false);
        value
    }
}
