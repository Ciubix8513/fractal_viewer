# fractal_viewer
A rust wgpu rewrite of my older [project](https://github.com/Ciubix8513/Fractals). Currently only supports native build, but I am planning to add wasm support.

## Currently supported fractals
 - Mandelbrot set (z^2 + c)  
 - Burning ship (abs(z)^2 + c)
 - Tricorn (conj(z)^2 + c)
 - Feather ((z^3 / 1 + z * z) + c)
 - Eye ((z/c)^2 - c)
