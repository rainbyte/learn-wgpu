# Rust and WGPU experiments

This repo has experiments about Rust and WGPU programming, including
samples from "Dr. Xu's series" and "Practical GPU Graphics" book.

Each program builds progressively to cover the following topics:

- Setting up a window
- Rendering a triangle
- Vertex and fragment shaders
- 3d model vertices and normals
- Rendering a cube
- Rotating a cube
- Blinn-Phong lightning
- Rotating cube with lightning
- Torus with lightning

## Features

- Migration to current `wgpu` and `winit` versions
- Tested on Linux wayland-based desktop environment
- Separate directories for each step

## Prerequisites

- Rust 2024 edition (installed via [rustup](https://rustup.rs))
- A modern GPU and updated drivers

## Getting Started

1. Clone the repository
2. Build

   ```sh
   cargo build
   ```

3. Run `cargo run` to launch a program, example:

   ```sh
   cargo run --bin wgpu12
   ```

Happy coding and GPU hacking! 🚀
