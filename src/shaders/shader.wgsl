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
    seed: f32,
    zoom: f32,
    x: f32,
    y: f32,
    maximum_iterations: u32,
    width: u32,
    height: u32,
    mu: f32,
    is_rendered: u32
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

fn itr(c: vec2<f32>,  z: vec2<f32>) -> f32{
    var i = 0;
    var z = z;
    while (length(z) < 8192. && i < 512) {
        z = vpow2(z) + c;
        i++;
    }
    return f32(i) + 1. - log(log(length(z))) / log(2.);
}



fn fcol(it: f32) -> vec3<f32> {
    if(it < 512.){
        return vec3(.5 + .5 * sin(it / 32.), .5 + .5 * sin(it / 48.), .5 + .5 * sin(it / 64.));
    }
    else{
        return vec3(0., 0., 0.);
    }
}

// cmul is a complex multiplication
fn cmul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    var pixel = vec2<u32>(
        u32((in.coord.x + 1.0) / 2.0 * f32(mandelbrot.width)),
        u32((in.coord.y + 1.0) / 2.0 * f32(mandelbrot.height))
    );
    var iteration = f32(mandelbrot.maximum_iterations);
    var cycle = iteration / 10.0;
    // create an array with a color lut
    // the lut is a gradient from black to white
    // the lut is used to color the mandelbrot set
    // the lut is created in the compute shader
    var lut: array<vec3<f32>, 100>;
    if(mandelbrot.is_rendered == 0u) {
        // draw a mandelbrot set
        // create a vec2 from the x0 and y0
        var seed = vec2<f32>(
            in.coord.x + mandelbrot.seed,
            in.coord.y + mandelbrot.seed
        );
        var r = fract(sin(dot(seed, vec2<f32>(12.9898, 78.233))) * 43758.5453);
        var fractalCoord = vec2(-1.1900443,0.3043895);
        var coord_x : f32 = mandelbrot.x;
        var coord_y : f32= mandelbrot.y;

        // create c from the x and y coordinates and by using the zoom with a constant heith / width ration, and the x and y coordinates of the mandelbrot set
        var c = vec2<f32>(coord_x, coord_y);
        var dc = vec2<f32>(
            in.coord.x * mandelbrot.zoom * f32(mandelbrot.width) / f32(mandelbrot.height) ,
            in.coord.y * mandelbrot.zoom
        );
        var z = vec2<f32>(0.0, 0.0);
        var dz = vec2<f32>(0.0, 0.0);
        var i = 0.0;
        var i2 = 0.0;
        var max = mandelbrot.mu;

        // calculate the iteration
        while (dot(dz, dz) < max && i < iteration) {
            dz = cmul(2.0 * z + dz,dz) + dc;
            z = vpow2(z) + c;
            i += 1.0;
        }
//        // add the rest to i to get a smooth color gradient
        i = i + 1.0 - log2(log2(dz.x * dz.x + dz.y * dz.y)) ;
        // normalize i to get a value between 0 and 1
        i = i / iteration;
        // calculate the iteration with the intensity
        mandelbrotTexture[ pixel.y * mandelbrot.width + pixel.x ] = i;
    }
    i = mandelbrotTexture[ pixel.y * mandelbrot.width + pixel.x ];
    // get the pixel coordinate
    var color = vec3<f32>(0.0, 0.0, 0.0);
    if (i < iteration) {

        var t = (i * cycle % cycle) ;
//        // calculate the color
//        var index = i % 100.0;
//        var index1 = u32(index);
//        var index2 = u32(index + 1.0) % 100u;
//        var t = index - f32(index1);
//        color = vec3<f32>(
//            mix(lut[index1].x, lut[index2].x, t),
//            mix(lut[index1].y, lut[index2].y, t),
//            mix(lut[index1].z, lut[index2].z, t)
//        );
//        // create a beautifull color gradient
        color = vec3<f32>(
            0.5 + 0.5 * cos(t * 6.28 + in.coord.x + mandelbrot.seed / 1000.0),
            0.5 + 0.5 * sin(t * 12.88 + sin(in.coord.y) + in.coord.y + mandelbrot.seed / 170.0),
            0.5 + 0.5 * cos(t * 3.14 + cos(in.coord.x * 3.14) + in.coord.y + mandelbrot.seed / 50.0)
        );
//        color = vec3<f32>(i / iteration , i / iteration , i / iteration);
//        color = vec3<f32>(i  , i  , i );
    }
    return vec4<f32>(color, 1.0);
}
