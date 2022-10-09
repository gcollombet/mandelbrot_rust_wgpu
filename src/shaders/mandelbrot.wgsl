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
    time_elapsed: f32,
    zoom: f32,
    center_delta: vec2<f32>,
    epsilon: f32,
    maximum_iterations: u32,
    width: u32,
    height: u32,
    mu: f32,
    must_redraw: u32,
    color_palette_scale: f32,
};

@group(0) @binding(0)
var<uniform> mandelbrot: Mandelbrot;

// add the storage buffer
@group(1) @binding(0)
var<storage, read_write> mandelbrotTexture: array<f32>;

// add the storage buffer
@group(2) @binding(0)
var<storage, read_write> mandelbrotOrbitPointSuite: array<vec2<f32>>;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.coord = model.coordinate.xy;
    return out;
}

// Fragment shader
fn vpow2(v: vec2<f32>) -> vec2<f32> {
     return vec2(v.x * v.x - v.y * v.y, 2. * v.x * v.y);
}

// cmul is a complex multiplication
fn cmul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

// create a function that colorize a pixel based on the number of iterations has seen below
fn colorize(coordinate: vec2<f32>, dc: vec2<f32>, iterations: f32) -> vec4<f32> {
    var cycle = mandelbrot.color_palette_scale;
    var color = vec3<f32>(0.0,0.0,0.0);
//    if(sqrt(iterations) % 2.0 < 1.0) {
        if(iterations < f32(mandelbrot.maximum_iterations)) {
            var log_iterations = sqrt(iterations);
            log_iterations = log_iterations * log_iterations;
            var t = abs(1.0 - (log_iterations % cycle) * 2.0 / cycle);
            // use a log scale to get a better color distribution
            color = vec3<f32>(
                0.5 + 0.5 * cos(t * 6.28 + coordinate.x + mandelbrot.time_elapsed / 3.3),
                0.5 + 0.5 * sin(t * 12.88 + sin(coordinate.y) + coordinate.y + mandelbrot.time_elapsed / 0.6),
                0.5 + 0.5 * cos(t * 3.14 + cos(coordinate.x * 3.14) + coordinate.y + mandelbrot.time_elapsed / 1.5)
            );
//            let wideness = 0.7;
////            let wideness = 0.2 + abs(cos(mandelbrot.time_elapsed / 4.0)) * 1.5;
//            var dx = coordinate.x / 8.0 + cos(mandelbrot.time_elapsed / 4.0);
////            dx= dx - modulo(dx, 3.14);
//            var dy = coordinate.y / 8.0 + cos(mandelbrot.time_elapsed / 8.0);
////            dy= dy - modulo(dy, 3.14);
//            color = vec3<f32>(
//                0.5 + 0.5 * cos(t * wideness * 6.28 + 1.4 + dx - 0.5 ),
//                0.5 + 0.5 * sin(t * wideness * 5.88 - 3.14 + sin(dy)),
//                0.5 + 0.5 * cos(t * wideness * 3.14 - 3.14 + cos(dx * 3.14) - 0.5)
//            );
        }
//    }
    return vec4<f32>(color, 1.0);
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
    if(mandelbrot.must_redraw == 0u) {
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
            }
            i += 1.0;
        }
        // add the rest to i to get a smooth color gradient
        let log_zn = log(dz.x * dz.x + dz.y * dz.y) / 2.0;
        var nu = log(log_zn / log(2.0)) / log(2.0);
        i += (1.0 - nu) ;
        // calculate the iteration with the intensity
        mandelbrotTexture[index] = i;
    }
    return color;
}
