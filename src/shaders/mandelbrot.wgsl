// Vertex shader

// TODO Compute color with a color lookup table
// TODO Give a special color to the outside color of the mandelbrot set (black) and on when espilon is reached (red)
// TODO Calculate a distance from the border when outside of the mandelbrot set
// TODO Calculate the distance from the border when inside the mandelbrot set https://www.shadertoy.com/view/lsX3W4
// TODO https://en.wikibooks.org/wiki/Fractals/Iterations_in_the_complex_plane/demm#Interior_distance_estimation
// TODO Render with max iterations 1000 and then render another 1000 in the remaning area
// TODO Use arbitraty precision number to calculate orbit

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) coordinate: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) coord: vec2<f32>,
};

// Define the uniform buffer from Mandelbrot Shader struct
struct Mandelbrot {
    generation: u32,
    time_elapsed: f32,
    zoom: f32,
    center_delta: vec2<f32>,
    epsilon: f32,
    maximum_iterations: u32,
    width: u32,
    height: u32,
    mu: f32,
    color_palette_scale: f32,
};

struct LastRenderedMandelbrot {
    center_delta: vec2<f32>,
    zoom: f32,
}


@group(0) @binding(0)
var<uniform> mandelbrot: Mandelbrot;
@group(0) @binding(1)
var<uniform> previous_mandelbrot: Mandelbrot;

// add the storage buffer
@group(0) @binding(2)
var<storage, read_write> mandelbrotTexture: array<f32>;
@group(0) @binding(3)
var<storage, read_write> previousMandelbrotTexture: array<f32>;
@group(0) @binding(4)
var<storage, read_write> mandelbrotZTexture: array<vec2<f32>>;

// add the storage buffer
@group(0) @binding(5)
var<storage, read_write> mandelbrotOrbitPointSuite: array<vec2<f32>>;
@group(0) @binding(6)
var<storage, read_write> lastRenderedMandelbrot: LastRenderedMandelbrot;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.coord = model.coordinate.xy;
    return out;
}

// cmul is a complex multiplication
fn cmul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

// create a function that colorize a pixel based on the number of iterations has seen below
fn colorize(coordinate: vec2<f32>, dc: vec2<f32>, iterations: f32) -> vec4<f32> {
    var color = vec4<f32>(0.0,0.0,0.0,0.0);
    if(
        mandelbrot.zoom != previous_mandelbrot.zoom
        || mandelbrot.center_delta.x != previous_mandelbrot.center_delta.x
        || mandelbrot.center_delta.y != previous_mandelbrot.center_delta.y
    ) {
        color = vec4<f32>(1.0,0.0,0.0,0.0);
    }
    if(iterations < f32(mandelbrot.maximum_iterations)) {
        var t = abs(1.0 - ((iterations + mandelbrot.time_elapsed * 5.0) % mandelbrot.color_palette_scale) * 2.0 / mandelbrot.color_palette_scale);
        var dx = coordinate.x / 8.0 + cos(mandelbrot.time_elapsed / 2.0);
        var dy = coordinate.y / 1.0 + cos(mandelbrot.time_elapsed / 2.0);
        color = vec4<f32>(
            0.5 + 0.5 * cos(t * 6.28 + 1.4 + sin(dx) - 0.5),
            0.5 + 0.5 * sin(t * 5.88 - 3.14 + sin(dy)),
            0.5 + 0.5 * cos(t * 3.14 - 3.14 + cos(dx * 3.14) - 0.5),
            1.0
        );
    }
    return color;
}

fn compute_iteration(dc: vec2<f32>, index: u32) {
    var iteration = f32(mandelbrot.maximum_iterations);
    // draw a mandelbrot set
    var z = mandelbrotOrbitPointSuite[0];
    var dz = vec2<f32>(0.0, 0.0);
    var i = 0.0;
    var max = mandelbrot.mu;
    // create an epsilon var that is smaller when the zoom is bigger
    var epsilon = mandelbrot.epsilon / pow(4.0, log2(1.0 / mandelbrot.zoom)) ;
    // calculate the iteration
    while (i < iteration) {
        z = mandelbrotOrbitPointSuite[u32(i)];
        dz = cmul(2.0 * z + dz,dz) + dc;
        mandelbrotZTexture[index] = dz;
        // if squared module of dz
        let dot_dz = dot(dz, dz);
         // if is bigger than a max value, then we are out of the mandelbrot set
        if (dot_dz >= max) {
            break;
        }
        //  if is lower then a epsilon value, then we are inside the mandelbrot set
        if (dot_dz < epsilon) {
            i = iteration;
            break;
        } else {
           i += 1.0;
        }

    }
    // add the rest to i to get a smooth color gradient
    let log_zn = log(dz.x * dz.x + dz.y * dz.y) / 2.0;
    var nu = log(log_zn / log(2.0)) / log(2.0);
    i += (1.0 - nu) ;
    // calculate the iteration with the intensity
    mandelbrotTexture[index] = i;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var pixel = vec2<u32>(
        u32((in.coord.x + 1.0) / 2.0 * f32(mandelbrot.width)),
        u32((in.coord.y + 1.0) / 2.0 * f32(mandelbrot.height))
    );
    var index = pixel.y * mandelbrot.width + pixel.x;
    var dc = vec2<f32>(
        mandelbrot.center_delta.x + in.coord.x * f32(mandelbrot.width) / f32(mandelbrot.height) * mandelbrot.zoom ,
        mandelbrot.center_delta.y + in.coord.y * mandelbrot.zoom
    );
    var color = colorize(in.coord, dc, mandelbrotTexture[index]);
    let movement = mandelbrot.center_delta - previous_mandelbrot.center_delta;
    let movement_x = movement.x / (f32(mandelbrot.width) / f32(mandelbrot.height)) / mandelbrot.zoom;
    let movement_y = movement.y / mandelbrot.zoom;
    if(movement_x != 0.0 || movement_y != 0.0) {
        let previous_pixel = vec2<i32>(
            i32((in.coord.x + movement_x + 1.0) / 2.0 * f32(mandelbrot.width)),
            i32((in.coord.y + movement_y + 1.0) / 2.0 * f32(mandelbrot.height))
        );
        if(
            u32(previous_pixel.x) < mandelbrot.width
            && u32(previous_pixel.y) < mandelbrot.height
            && previous_pixel.x > 0
            && previous_pixel.y > 0
        ) {
            let previous_index = u32(previous_pixel.y) * mandelbrot.width + u32(previous_pixel.x);
            mandelbrotTexture[index] = previousMandelbrotTexture[previous_index];
        } else {
            compute_iteration(dc, index);
            return color;
        }
    }
    if(
        mandelbrot.zoom != previous_mandelbrot.zoom
    ) {
        // a var that contain the norm of the in.coord vector
        let norm = sqrt(in.coord.x * in.coord.x + in.coord.y * in.coord.y);
        // make the norm follow a square curve
        let norm_square = 1u + u32(norm * norm * 50.0);
        if(
           !(pixel.x % norm_square == (mandelbrot.generation % norm_square))
        && !(pixel.y % norm_square == (mandelbrot.generation % norm_square))
        ) {
            let zoom_factor = mandelbrot.zoom / previous_mandelbrot.zoom;
            let previous_pixel = vec2<i32>(
                i32((in.coord.x * zoom_factor + 1.0) / 2.0 * f32(mandelbrot.width)),
                i32((in.coord.y * zoom_factor + 1.0) / 2.0 * f32(mandelbrot.height))
            );
            if(
                u32(previous_pixel.x) < (mandelbrot.width - 2u)
                && u32(previous_pixel.y) < (mandelbrot.height - 2u)
                && previous_pixel.x > 2
                && previous_pixel.y > 2
            ) {
                let previous_index = u32(previous_pixel.y) * mandelbrot.width + u32(previous_pixel.x);
                // mandelbrotTexture[index] equal a sample of the five pixel around the previous pixel
                // if the pixels are > max_iteration, then use the previous pixel
                // else use the average of the five pixel
                let previous_iteration = previousMandelbrotTexture[previous_index];
//                let previous_iteration_1 = previousMandelbrotTexture[previous_index + 1u];
//                let previous_iteration_2 = previousMandelbrotTexture[previous_index + mandelbrot.width];
//                let previous_iteration_3 = previousMandelbrotTexture[previous_index + mandelbrot.width + 1u];
//                let previous_iteration_4 = previousMandelbrotTexture[previous_index + mandelbrot.width - 1u];
//                let previous_iteration_5 = previousMandelbrotTexture[previous_index - mandelbrot.width];
//                let previous_iteration_6 = previousMandelbrotTexture[previous_index - mandelbrot.width + 1u];
//                let previous_iteration_7 = previousMandelbrotTexture[previous_index - mandelbrot.width - 1u];
//                let previous_iteration_8 = previousMandelbrotTexture[previous_index - 1u];
                let iteration = f32(previous_mandelbrot.maximum_iterations);
//                if(
//                    previous_iteration >= iteration
//                    || previous_iteration_1 > iteration
//                    || previous_iteration_2 > iteration
//                    || previous_iteration_3 > iteration
//                    || previous_iteration_4 > iteration
//                    || previous_iteration_5 > iteration
//                    || previous_iteration_6 > iteration
//                    || previous_iteration_7 > iteration
//                    || previous_iteration_8 > iteration
//                ) {
                 mandelbrotTexture[index] = previous_iteration;
//                } else {
//                    mandelbrotTexture[index] = (
//                    previous_iteration
//                    + previous_iteration_1
//                    + previous_iteration_2
//                    + previous_iteration_3
//                    + previous_iteration_4
//                    + previous_iteration_5
//                    + previous_iteration_6
//                    + previous_iteration_7
//                    + previous_iteration_8
//                    ) / 18.0;
//                }
            } else {
                compute_iteration(dc, index);
            }
        } else {
            compute_iteration(dc, index);
        }
    }
    return color;
}
