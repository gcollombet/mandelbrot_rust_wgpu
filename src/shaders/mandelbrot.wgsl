// Vertex shader

// TODO Compute color with a color lookup table
// TODO Give a special color to the outside color of the mandelbrot set (black) and on when espilon is reached (red)
// TODO Calculate a distance from the border when outside of the mandelbrot set
// TODO Calculate the distance from the border when inside the mandelbrot set https://www.shadertoy.com/view/lsX3W4
// TODO https://en.wikibooks.org/wiki/Fractals/Iterations_in_the_complex_plane/demm#Interior_distance_estimation
// TODO Render with max iterations 1000 and then render another 1000 in the remaning area
// TODO Use arbitraty precision number to calculate orbit
// TODO https://www.shadertoy.com/view/wdBfDK Smart AA
// TODO https://www.shadertoy.com/view/4sdXWX
// TODO https://www.shadertoy.com/view/lsX3W4 Estimation de la distance à la frontière
// TODO https://www.shadertoy.com/view/ldf3DN Orbit traps
// TODO infinite zoom https://www.shadertoy.com/view/7ly3Wh
// TODO https://www.shadertoy.com/view/NtKXRy Infinite zoom
// TODO https://fractalforums.org/fractal-mathematics-and-new-theories/28/another-solution-to-perturbation-glitches/4360
// TODO https://fractalforums.org/fractal-mathematics-and-new-theories/28/another-solution-to-perturbation-glitches/4360/60
// TODO https://mathr.co.uk/blog/2021-05-14_deep_zoom_theory_and_practice.html
// TODO https://fractalforums.org/fractal-mathematics-and-new-theories/28/another-solution-to-perturbation-glitches/4360/90
// TODO https://code.mathr.co.uk/mandelbrot-numerics/blob/HEAD:/c/bin/m-describe.c
// https://randomascii.wordpress.com/2012/01/11/tricks-with-the-floating-point-format/ About Floating point
// https://fractalforums.org/fractal-mathematics-and-new-theories/28/criteria-for-sa-stop/4628/msg32837#msg32837
// http://www.science.eclipse.co.uk/sft_maths.pdf
// https://mathr.co.uk/mandelbrot/book-draft/
// https://mathr.co.uk/blog/2010-08-31_optimizing_zoom_animations.html
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
    // the number of frame rendered so far
    generation: u32,
    // the time elapsed since the start of the program
    time_elapsed: f32,
    // the zoom factor
    zoom: f32,
    angle: f32,
    // the delta between the the dc dot and the  mandelbrot coordinate at the screen center
    center_delta: vec2<f32>,
    epsilon: f32,
    maximum_iterations: u32,
    // the width in pixel of the screen
    width: u32,
    // the height in pixel of the screen
    height: u32,
    // the maximum value to consider the point is in the mandelbrot set
    mu: f32,
    // the color palette scale factor
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
var<storage, read_write> mandelbrotData: array<vec2<f32>>;
@group(0) @binding(5)
var<storage, read_write> previousMandelbrotData: array<vec2<f32>>;

// add the storage buffer
@group(0) @binding(6)
var<storage, read_write> mandelbrotOrbitPointSuite: array<vec2<f32>>;
@group(0) @binding(7)
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

// Fragment shader
fn vpow2(v: vec2<f32>) -> vec2<f32> {
     return vec2(v.x * v.x - v.y * v.y, 2. * v.x * v.y);
}

// cmul is a complex multiplication
fn cmul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

// cdiv is a complex division
fn cdiv(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    var denominator: f32 = b.x * b.x + b.y * b.y;
    return vec2<f32>((a.x * b.x + a.y * b.y) / denominator, (a.y * b.x - a.x * b.y) / denominator);
}

// create a function that colorize a pixel based on the number of iterations has seen below
fn colorize(coordinate: vec2<f32>, dc: vec2<f32>, iterations: f32, derivative: vec2<f32>) -> vec4<f32> {
    var color = vec4<f32>(0.0,0.0,0.0,1.0);
    if(iterations >= 0.0) {
        var t = abs(1.0 - ((iterations + mandelbrot.time_elapsed * 5.0) % mandelbrot.color_palette_scale) * 2.0 / mandelbrot.color_palette_scale);
        var dx = coordinate.x / 5.0;
        var dy = coordinate.y / 5.0;
        color = vec4<f32>(
            0.5 + 0.5 * cos(t *  6.28 + 1.4 + sin(dx) - 0.5),
            0.5 + 0.5 * sin(t * 5.88 - 3.14 + sin(dy - dx)),
            0.5 + 0.5 * cos(t * 3.14 - 3.14 + cos(dx * 3.14) - 0.5),
            1.0
        );
        // multiply the color by the phong shading using the derivative
        // the light is rotated around the z axis to give a nice effect
        var light = normalize(vec3<f32>(cos(mandelbrot.time_elapsed * 0.5), sin(mandelbrot.time_elapsed * 0.5), 1.5));
        var normal = normalize(vec3<f32>(derivative.x, derivative.y, 1.0));
        var diffuse = min(max(dot(normal, light), 0.2) * 2.5,1.0);
        color = vec4<f32>(color.rgb * diffuse , 1.0);
    } else {
        color = vec4<f32>(0.0,0.0,0.0,1.0);
    }
    return color;
}

fn compute_iteration(dc: vec2<f32>, index: u32, max_iteration: u32) -> f32 {
    var max_iteration: f32 = f32(max_iteration);
    // draw a mandelbrot set
    var z = mandelbrotOrbitPointSuite[0];
    var dz = vec2<f32>(0.0, 0.0);
    var der = vec2<f32>(1.0, 0.0);
    var distance = 0.0;
    var i = 0.0;
    var ref_i = 0;
    var max = mandelbrot.mu;
    // create an epsilon var that is smaller when the zoom is bigger
    var epsilon = mandelbrot.epsilon;
    // calculate the iteration
    while (i < max_iteration) {
        z = mandelbrotOrbitPointSuite[ref_i];
        dz = 2.0 * cmul(dz, z) + cmul(dz, dz) + dc;
        ref_i += 1;
        // if squared module of dz
        z = mandelbrotOrbitPointSuite[ref_i] + dz;
        mandelbrotData[index] = cdiv(der,z);
        let dot_z = dot(z, z);
         // if is bigger than a max value, then we are out of the mandelbrot set
        if (dot_z >= max) {
            break;
        }
        if (dot(der, der) < epsilon) {
            i = -3.0;
            break;
        }
        der = cmul(der * 2.0, z);
        let dot_dz = dot(dz, dz);
        if (dot_z < dot_dz || f32(ref_i) == max_iteration) {
            dz = z;
            ref_i = 0;
        } else {
//   /* bivariate linear approximation */
//    T := lookup table [ exponent(|z|^2) ]
//    z := T.U * z + T.V * c + T.W
//    iterations += T.iterations
//    reference iterations += T.iterations
        }
        i += 1.0;
    }
    if(i >= max_iteration ) {
        i = -1.0;
    } else {
        if( i > 0.0) {
            // add the rest to i to get a smooth color gradient
            let log_zn = log(dz.x * dz.x + dz.y * dz.y) / 2.0;
            var nu = log(log_zn / log(2.0)) / log(2.0);
            i += (1.0 - nu) ;
        }
    }
    return i;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // make a random number between 0 and 1 from mandelbrot.generation
    let random = fract(sin(f32(mandelbrot.generation) * 12.9898) * 43758.5453);
    var pixel = vec2<u32>(
        u32((in.coord.x + 1.0) / 2.0 * f32(mandelbrot.width)),
        u32((in.coord.y + 1.0) / 2.0 * f32(mandelbrot.height))
    );
    let screen_ratio = f32(mandelbrot.width) / f32(mandelbrot.height);
    var index = pixel.y * mandelbrot.width + pixel.x;
    var coord = in.coord;
    // scale the coord with zoom
    coord = coord * mandelbrot.zoom;
    // rotate the coord
    coord.x *= screen_ratio;
    coord = vec2<f32>(
        coord.x * cos(mandelbrot.angle) - coord.y * sin(mandelbrot.angle),
        coord.x * sin(mandelbrot.angle) + coord.y * cos(mandelbrot.angle)
    );
    var dc = vec2<f32>(
        mandelbrot.center_delta.x + coord.x,
        mandelbrot.center_delta.y + coord.y
    );
    var movement = mandelbrot.center_delta - previous_mandelbrot.center_delta;
    movement.x = movement.x / (f32(mandelbrot.width) / f32(mandelbrot.height)) / mandelbrot.zoom;
    movement.y = movement.y / mandelbrot.zoom;
    if(
        mandelbrot.zoom != previous_mandelbrot.zoom
        || mandelbrot.angle != previous_mandelbrot.angle
        || movement.x != 0.0
        || movement.y != 0.0
    ) {
        // a var that contain the norm of the in.coord vector
        let norm = sqrt(in.coord.x * in.coord.x + in.coord.y * in.coord.y);
        // the norm of mandelbrot width height
        let norm_mandelbrot = sqrt(f32(mandelbrot.width) * f32(mandelbrot.width) + f32(mandelbrot.height) * f32(mandelbrot.height));
        // make the norm follow a square curve
        let norm_square = 1u + u32(norm * norm * norm_mandelbrot / 100.0);
        let zoom_factor = mandelbrot.zoom / previous_mandelbrot.zoom;
        // calculat angle delta from previous_mandelbrot.angle and mandelbrot.angle
        // angle_delta vari between 0 and 2 pi
        let angle_delta = mandelbrot.angle - previous_mandelbrot.angle;
        // scale coord by zoom_factor
        var coord = in.coord;
        // scale coord by zoom_factor
        coord *= zoom_factor;
        // rotate coord by angle_delta
        coord.x *= screen_ratio;
        coord = vec2<f32>(
            coord.x * cos(angle_delta) - coord.y * sin(angle_delta),
            coord.x * sin(angle_delta) + coord.y * cos(angle_delta)
        );
        coord.x /= screen_ratio;
        // rotate movement by angle
        movement.x *= screen_ratio;
        movement = vec2<f32>(
            movement.x * cos(-mandelbrot.angle) - movement.y * sin(-mandelbrot.angle),
            movement.x * sin(-mandelbrot.angle) + movement.y * cos(-mandelbrot.angle)
        );
        movement.x /= screen_ratio;
        // translate coord by movement
        coord += movement;
       // calculate the new pixel
        var previous_pixel = vec2<f32>(
            (coord.x + 1.0) / 2.0 * f32(mandelbrot.width),
            (coord.y + 1.0) / 2.0 * f32(mandelbrot.height)
        );
        let previous_index = u32(previous_pixel.y) * mandelbrot.width + u32(previous_pixel.x);
        if(
           mandelbrot.angle == previous_mandelbrot.angle
           && (
               mandelbrot.zoom == previous_mandelbrot.zoom
               || (
                 !(pixel.x % norm_square == u32(random * f32(norm_square)))
                 && !(pixel.y % norm_square == u32(random * f32(norm_square)))
               )
           )
        ) {
            if(
                previous_pixel.x < f32(mandelbrot.width)
                && previous_pixel.y < f32(mandelbrot.height)
                && previous_pixel.x >= 0.0
                && previous_pixel.y >= 0.0
            ) {
//                mandelbrotTexture[index] = previousMandelbrotTexture[previous_index];
                mandelbrotTexture[index] = previousMandelbrotTexture[previous_index];
                mandelbrotData[index] = previousMandelbrotData[previous_index];
            } else {
                mandelbrotTexture[index] = compute_iteration(dc, index, mandelbrot.maximum_iterations);
            }
        } else {
            mandelbrotTexture[index] = compute_iteration(dc, index, mandelbrot.maximum_iterations);
        }
    }
    return colorize(in.coord, dc, mandelbrotTexture[index], mandelbrotData[index]);
}
