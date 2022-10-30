
# Real time Mandelbrot set explorer

It has been done millions of times, but I wanted to do it myself. This version is focused on real time navigation.

The acceleration given by the GPU allow to render the Mandelbrot set in real time, even at high zoom level.

The navigation is done with the mouse, and the zoom is done with the mouse wheel.

### Perturbation theory

This mandelbrot explorer uses perturbation theory to render the Mandelbrot set at a great zoom level, but not infinite.

I do understand, vaguely, the concept [thanks to this guy](https://www.youtube.com/playlist?list=PL43B1963F261E6E47).

But obviously, I'm unable to use it, and I was unaware of it before I read a post made by smarter guys than me on Fractal Forum.

There, they discuss the fact that one could use perturbation theory and apply it to mandelbrot set formula 
to mitigate precisions issues when zooming in. They also provide the associated formula and even a pseudocode implementation.

The core idea that this technic is that it allow to make calculus with numbers really close to zero, 
where the precision of floating point is the greatest. 

It is particularly important when the calculation is done by GPU because they do work with 32 bits floating point numbers.

With perturbation, the zoom limit is around 10^-40, so very close to the smallest positive number of single precision floating point limit at 10^-45.

## Controls

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

## Download

Download the latest release [here](https://github.com/gcollombet/mandelbrot_rust_wgpu/releases/latest).







