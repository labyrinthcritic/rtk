# rtk

<div align="center">
  <img src="docs/plane_with_spheres.png" height="250px">
  <img src="docs/cornell_box.png" height="250px">
  <br />
  ray-tracing kiwis
  <br />
</div>

This is a software ray-tracer based on the books at [raytracing.github.io](https://raytracing.github.io).

## Features

- Render a scene with its camera, objects, and materials described in a toml file.
  See [examples](examples).
- Parallelized with `rayon`.
  - In testing, `examples/plane_with_spheres.toml` rendered in 7 minutes with `--no-parallel`
    and 70 seconds with `--parallel`, a 6x speed-up.

## Usage

rtk is an ordinary Cargo project. Run `cargo build --release` or `cargo install --path .`.

To render the Cornell box example:

```sh
/target/release/rtk examples/cornell_box.toml
```
