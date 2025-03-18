## Requirements

- Godot 4.2.2+
- Rust 1.27.0+

## Tech stack

- [Godot Engine](https://docs.godotengine.org/en/stable/)
- [Rapier](https://rapier.rs/docs)
- [godot-rust (Rust bindings for GDExtension)](https://godot-rust.github.io/)

## Guidelines

- Follow [semver](https://semver.org/) when releasing new versions
- If functionality doesn't need to directly interact with Rapier and doesn't need to be optimized, prefer writing it in GDScript
- Any mutations to the Rapier pipeline need to be fed through the action queue in order to ensure determinism

## Development quickstart

1. Open this project in Godot
1. Make edits as desired
1. Run `cargo build` in the `/addons/godot-rapier-3d/rust` directory
1. The extension should automatically reload when Godot is refocused

## Bugs

Please raise an issue and provide reproducible steps or a minimal reproduction project, which is a small Godot project which reproduces the issue, with no unnecessary files included.

## WASM build

Use emscripten version 3.1.74

To build the wasm binaries locally use these commands (refer to https://github.com/deltasiege/godot-rapier-3d/blob/main/.github/workflows/jobs-build.yml if these are outdated)

```bash
# Multi-threaded
set RUSTFLAGS=-C link-arg=-fwasm-exceptions -Cllvm-args=-wasm-enable-sjlj -C link-args=-sDISABLE_EXCEPTION_CATCHING=1 -C link-args=-sSUPPORT_LONGJMP=wasm -C link-args=-pthread -C link-args=-sSIDE_MODULE=2  -C target-feature=+atomics,+bulk-memory,+mutable-globals -Zlink-native-libraries=no -Cllvm-args=-enable-emscripten-cxx-exceptions=0
cargo +nightly build -Zbuild-std --target wasm32-unknown-emscripten
mv ./target/wasm32-unknown-emscripten/debug/godot_rapier_3d.wasm ./target/wasm32-unknown-emscripten/debug/godot_rapier_3d.threads.wasm

# Single-threaded
set RUSTFLAGS=-C link-arg=-fwasm-exceptions -Cllvm-args=-wasm-enable-sjlj -C link-args=-sDISABLE_EXCEPTION_CATCHING=1 -C link-args=-sSUPPORT_LONGJMP=wasm -C link-args=-sSIDE_MODULE=2 -C target-feature=+atomics,+bulk-memory,+mutable-globals -Zlink-native-libraries=no -Cllvm-args=-enable-emscripten-cxx-exceptions=0
cargo +nightly build --features nothreads -Zbuild-std --target wasm32-unknown-emscripten
```

You should end up with 2 .wasm binaries in `target/debug` after running the above

<!-- https://discord.com/channels/723850269347283004/1351083593144733696/1351411672820219904 -->
