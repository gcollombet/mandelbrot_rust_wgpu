
# Realtime Mandelbrot set explorer

Yet another mandelbrot set explorer.

![Cover](/assets/illustration_2.png)
![Cover](/assets/illustration_3.png)
![Cover](/assets/illustration_7.png)
![Cover](/assets/illustration.png)
![Cover](/assets/illustration_6.png)
![Cover](/assets/illustration_5.png)
![Cover](/assets/illustration_4.png)
![Cover](/assets/illustration_8.png)


It has been done millions of times, but I wanted to do it myself... This version is focused on real time navigation.

The navigation is made with the mouse, and the zoom is made with the mouse wheel.

The acceleration given by the GPU allows to render the Mandelbrot set in real time, even at high zoom level.

## Use it

### Download

Download the latest release [here](https://github.com/gcollombet/mandelbrot_rust_wgpu/releases/latest).

### Controls

- `Mouse wheel` to zoom at center of screen
- `Left mouse pressed` to move
- `Right mouse pressed` to rotate
- Arrow keys or `Z`, `Q`, `S`, `D` to move
- `A` and `E` to rotate left and right
- Numpad `+` and `-` to change the zoom speed
- Numpad `/` and `*` to change the iteration count
- `Space` pause the animation
- `Entrer` to reset the zoom and rotation
- `Page up/down` to increase/decrease the color palette scale
- `F11` to toggle fullscreen
- `Escape` to quit

## General information

### Perturbation theory

This mandelbrot explorer uses perturbation theory to render the Mandelbrot set at a great zoom level, but not infinite.

I do understand, vaguely, the concept [thanks to this guy](https://www.youtube.com/playlist?list=PL43B1963F261E6E47).

But obviously, I'm unable to use it, and I was unaware of it before 
I read [a post made by smarter guys than me on Fractal Forum](https://fractalforums.org/fractal-mathematics-and-new-theories/28/another-solution-to-perturbation-glitches/4360).

There, they discuss the fact that one could use perturbation theory and apply it to mandelbrot set formula 
to mitigate precision issues when zooming in. They also provide the associated formula and even a pseudocode implementation.

The core idea of this technic is that it allow to make calculus with numbers really close to zero, 
[where the floating point precision is greatest](https://randomascii.wordpress.com/2012/01/11/tricks-with-the-floating-point-format/). 

It is particularly important when the calculation is done by GPU because they do work with 32 bits floating point numbers.

With perturbation, the zoom limit is around 10^-40, which is very close to the smallest positive number of single precision floating point limit at 10^-45.

### Optimizations

When zooming in or moving, only the part of the image that has changed is rendered.

The iteration count is automatically increased when zooming in, and decreased when zooming out.

The mandelbrot iteration calculus loop is escaped when the derivative of z is close to an arbitrary epsilon threshold.
