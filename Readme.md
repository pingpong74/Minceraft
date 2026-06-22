# Minceraft

Minceraft is a voxel engine inspired by Minecraft, written in Rust and rendered using Vulkan.

The project focuses on modern rendering techniques and efficient world streaming rather than gameplay features. Chunks are generated asynchronously on the CPU, while rendering is driven almost entirely by the GPU to minimize CPU overhead.

## Features

* Vulkan-based renderer
* Multithreaded chunk generation
* Frustum culling
* Z-prepass rendering
* GPU-driven indirect drawing
* Chunk streaming and unloading
* Cross-platform Rust codebase

## Goals

This project serves as a playground for experimenting with:

* Modern graphics programming
* GPU-driven rendering pipelines
* Vulkan abstractions and task graphs
* Large-scale voxel world rendering
* Multithreaded engine architecture

## Building

Note: slangc has to be installed.

```bash
cargo run --release
```
