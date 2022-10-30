
# Real time Mandelbrot set explorer

It has been done millions of times, but I wanted to do it myself. This version is focused on real time navigation.

This mandelbrot explorer uses perturbation theory to render the Mandelbrot set at a great zoom level, but not infinite.
The zoom limit is around 10^-40, so very close to the smallest positive number of single precision floating point limit.

The acceleration given by the GPU allow to render the Mandelbrot set in real time, even at high zoom level.

The navigation is done with the mouse, and the zoom is done with the mouse wheel.

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







