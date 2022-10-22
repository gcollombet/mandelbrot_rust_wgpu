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
    var color = vec4<f32>(0.0,0.0,0.0,1.0);
    if(iterations < f32(mandelbrot.maximum_iterations)) {
        var t = abs(1.0 - ((iterations + mandelbrot.time_elapsed * 5.0) % mandelbrot.color_palette_scale) * 2.0 / mandelbrot.color_palette_scale);
        var dx = coordinate.x / 1.0 + cos(mandelbrot.time_elapsed / 2.0);
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
    if(i >= iteration) {
        i = 100000.0;
    } else {
        // add the rest to i to get a smooth color gradient
        let log_zn = log(dz.x * dz.x + dz.y * dz.y) / 2.0;
        var nu = log(log_zn / log(2.0)) / log(2.0);
        i += (1.0 - nu) ;
    }
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
        let norm_square = 1u + u32(norm * norm * f32(mandelbrot.width) / 10.0);
        if(
           !(pixel.x % norm_square == (mandelbrot.generation % norm_square))
        && !(pixel.y % norm_square == (mandelbrot.generation % norm_square))
        ) {
            let zoom_factor = mandelbrot.zoom / previous_mandelbrot.zoom;
            let previous_pixel = vec2<f32>(
                (in.coord.x * zoom_factor + 1.0) / 2.0 * f32(mandelbrot.width),
                (in.coord.y * zoom_factor + 1.0) / 2.0 * f32(mandelbrot.height)
            );
            if(
                previous_pixel.x < f32(mandelbrot.width)
                && previous_pixel.y < f32(mandelbrot.height)
                && previous_pixel.x > 0.0
                && previous_pixel.y > 0.0
            ) {

                //calculate index of surrounding pixels using bilinear interpolation to get a smooth zoom
                // interpolate between the 9 surrounding pixels
//                let x = fract(previous_pixel.x);
//                let y = fract(previous_pixel.y);
//                let x00 = floor(previous_pixel.x);
//                let y00 = floor(previous_pixel.y);
//                let x01 = x00 + 1.0;
//                let y01 = y00 + 1.0;
//                let x10 = x00 - 1.0;
//                let y10 = y00 - 1.0;
//                let index0 = u32(y00) * mandelbrot.width + u32(x00);
//                let index1 = u32(y01) * mandelbrot.width + u32(x01);
//                let index2 = u32(y00) * mandelbrot.width + u32(x01);
//                let index3 = u32(y10) * mandelbrot.width + u32(x01);
//                let index4 = u32(y01) * mandelbrot.width + u32(x00);
//                let index5 = u32(y10) * mandelbrot.width + u32(x00);
//                let index6 = u32(y01) * mandelbrot.width + u32(x10);
//                let index7 = u32(y00) * mandelbrot.width + u32(x10);
//                let index8 = u32(y10) * mandelbrot.width + u32(x10);
//                let _i0 = previousMandelbrotTexture[index0];
//                let _i1 = previousMandelbrotTexture[index1];
//                let _i2 = previousMandelbrotTexture[index2];
//                let _i3 = previousMandelbrotTexture[index3];
//                let _i4 = previousMandelbrotTexture[index4];
//                let _i5 = previousMandelbrotTexture[index5];
//                let _i6 = previousMandelbrotTexture[index6];
//                let _i7 = previousMandelbrotTexture[index7];
//                let _i8 = previousMandelbrotTexture[index8];
////                let i = mix(mix(i1, i2, x), mix(i3, i4, x), y);
//                let ir = mix(_i0, _i7, x);
//                let il = mix(_i0, _i7, 1.0 - x);
//                let it = mix(_i0, _i4, y);
//                let ib = mix(_i0, _i5, 1.0 - y);
//                let itr = mix(_i0, _i1, x);
//                let itl= mix(_i0, _i6, x);
//                let itt = mix(itr, itl, x);
//                let ibr= mix(_i0, _i3, x);
//                let ibl= mix(_i0, _i8, x);
//                let ibb = mix(ibr, ibl, x);
//                let i = mix(itt, ibb, y);
//                mandelbrotTexture[index] = (_i0 + i + it + ib + ir + il) / 6.0;

                let x0 = u32(floor(previous_pixel.x));
                let x1 = x0 + 1u;
                let y0 = u32(floor(previous_pixel.y));
                let y1 = y0 + 1u;
                let x = previous_pixel.x - f32(x0);
                let y = previous_pixel.y - f32(y0);
                let index00 = y0 * mandelbrot.width + x0;
                let index01 = y1 * mandelbrot.width + x0;
                let index10 = y0 * mandelbrot.width + x1;
                let index11 = y1 * mandelbrot.width + x1;
                let i00 = previousMandelbrotTexture[index00];
                let i01 = previousMandelbrotTexture[index01];
                let i10 = previousMandelbrotTexture[index10];
                let i11 = previousMandelbrotTexture[index11];
                let i0 = mix(i00, i10, x);
                let i1 = mix(i01, i11, x);
                let i = mix(i0, i1, y);
//
//                mandelbrotTexture[index] = i;
            } else {
                // le cas du dézoom
//                if(mandelbrot.generation % 10u == 0u) {
//                    compute_iteration(dc, index);
//                }
            }
        } else {
//            let previous_index = u32(previous_pixel.y) * mandelbrot.width + u32(previous_pixel.x);
            compute_iteration(dc, index);
//            mandelbrotTexture[index] = (mandelbrotTexture[index] + previousMandelbrotTexture[previous_index]) / 2.0;
        }
    }
    return color;
}
