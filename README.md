# Fantasy Crescendo... in Bevy

Because Unity's Entities/DOTS has taken years to deliver on what the Rust
community is actively engaged in developing and in the interest of entirely open
source game development, this is an experimental reimplementation of the core
gameplay items of Fantasy Crescendo.

## Goals

 * Create a stable cross-platform determinmistic simulation for rollback netplay.
 * Get lower level control over the game's systems.
 * Deliver a custom cross-platform editor that can more easily create the game's
   assets.

## Future Direction

This is still an experiment. It's unknown if this implementation is going to
supplant the Unity implementation, be used as a simulation layer, or whatever
else.

## Development Environment Setup

Local development environment requires the following:
 
 * [rust/cargo 1.52+](https://www.rust-lang.org/tools/install)
 * [LLVM/Clang 12.0.0](https://github.com/llvm/llvm-project/releases/tag/llvmorg-12.0.0)

The project currently will only build on nightly due to the use of an unstable cargo 
feature that has not landed in stable yet. Use `rustup` to switch to nightly Rust.