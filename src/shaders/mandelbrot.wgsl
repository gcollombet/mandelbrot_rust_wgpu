// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) coord: vec2<f32>,
};

// Define the uniform buffer from Mandelbrot Shader struct
struct MandelbrotUniform {
    generation : f32,
    zoom: f32,
    center_coordinate: vec2<f32>,
    near_orbit_coordinate: vec2<f32>,
    epsilon: f32,
    maximum_iterations: u32,
    width: u32,
    height: u32,
    mu: f32,
    must_redraw: u32,
    color_palette_scale: u32,
};

@group(0) @binding(0)
var<uniform> mandelbrot: MandelbrotUniform;

// add the storage buffer
@group(1) @binding(0)
var<storage, read_write> mandelbrotTexture: array<f32>;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.coord = model.position.xy;
    return out;
}

// Fragment shader

fn vpow2(v: vec2<f32>) -> vec2<f32> {
     return vec2(v.x * v.x - v.y * v.y, 2. * v.x * v.y);
}

fn rv( v: vec2<f32>, r: f32) -> vec2<f32> {
    var a: f32 = atan2(v.y, v.x);
    a += r;
    return vec2(cos(a), sin(a)) * length(v);
}


// cmul is a complex multiplication
fn cmul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

// create a function that colorize a pixel based on the number of iterations has seen below
fn colorize(coordinate: vec2<f32>, iterations: f32) -> vec4<f32> {
    var cycle = f32(mandelbrot.color_palette_scale);
    var color = vec3<f32>(0.0,0.0,0.0);
    if(iterations < f32(mandelbrot.maximum_iterations)) {
        var log_iterations = log2(iterations);
        log_iterations = log_iterations * log_iterations;
        var t = abs(1.0 - (log_iterations % cycle) * 2.0 / cycle);
        // use a log scale to get a better color distribution
        color = vec3<f32>(
            0.5 + 0.5 * cos(t * 6.28 + coordinate.x + mandelbrot.generation / 1000.0),
            0.5 + 0.5 * sin(t * 12.88 + sin(coordinate.y) + coordinate.y + mandelbrot.generation / 170.0),
            0.5 + 0.5 * cos(t * 3.14 + cos(coordinate.x * 3.14) + coordinate.y + mandelbrot.generation / 50.0)
        );
    }
    return vec4<f32>(color, 1.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    var pixel = vec2<u32>(
        u32((in.coord.x + 1.0) / 2.0 * f32(mandelbrot.width)),
        u32((in.coord.y + 1.0) / 2.0 * f32(mandelbrot.height))
    );
    let index = pixel.y * mandelbrot.width + pixel.x;
    var iteration = f32(mandelbrot.maximum_iterations);
    if(mandelbrot.must_redraw == 0u) {
        // draw a mandelbrot set

        var c = mandelbrot.near_orbit_coordinate;
        var dc = vec2<f32>(
            (mandelbrot.center_coordinate.x - c.x) + in.coord.x * mandelbrot.zoom * f32(mandelbrot.width) / f32(mandelbrot.height),
            (mandelbrot.center_coordinate.y - c.y) + in.coord.y * mandelbrot.zoom
        );
        var z = vec2<f32>(0.0, 0.0);
        var dz = vec2<f32>(0.0, 0.0);
//        var ddz = vec2<f32>(1.0, 0.0);
        var i = 0.0;
        var max = mandelbrot.mu;
        // create an epsilon var that is smaller when the zoom is bigger

        var epsilon = mandelbrot.epsilon;
        if(mandelbrot.zoom < 1.0) {
            epsilon = mandelbrot.epsilon / pow(10.0, log2(1.0 / mandelbrot.zoom)) ;
        }
        // calculate the iteration
        while (i < iteration) {
//            ddz = ddz * 2.0 * z + vec2<f32>(1.0, 0.0);
            dz = cmul(2.0 * z + dz,dz) + dc;
            // if squared module of dz is lower then a epsilon value, then break the loop
            if (dot(dz, dz) >= max) {
                break;
            }
            z = vpow2(z) + c;
            if (dot(z,z) < epsilon) {
                i = iteration;
                break;
            } else {
                i += 1.0;
            }
        }

//        var v = vec2<f32>(0.5, 0.5);
//        let h2 = 1.5;
//        var u = z / dz;
//        u = u / abs(u);
//        var t = u.x * v.x + u.y * v.y + h2;
//        t = t / (1.0 + h2);
//        if (t < 0.0) {
//            t = 0.0;
//        }

        // add the rest to i to get a smooth color gradient
        let log_zn = log(dz.x * dz.x + dz.y * dz.y) / 2.0;
        var nu = log(log_zn / log(2.0)) / log(2.0);
        i += (1.0 - nu) ;


        // calculate the iteration with the intensity
        mandelbrotTexture[index] = i;
    }
    i = mandelbrotTexture[index];
    return colorize(in.coord, i);
}
