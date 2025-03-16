Determinism diffs are uploaded with each release by the [Github Actions workflow](https://github.com/deltasiege/godot-rapier-3d/actions/workflows/build-and-test.yml)

[Download the latest diffs](https://github.com/deltasiege/godot-rapier-3d/releases/latest/download/determinism-diffs.zip)

These are obtained by hashing the entire physics simulation each frame on 2 different machines, and then comparing the hashes.

In theory, every hash should always match, even across different OSes & CPU architectures

## Currently tested targets

Each target is compared to every other target

| Linux                     | Windows                | MacOS                |
| ------------------------- | ---------------------- | -------------------- |
| x86_64-unknown-linux-gnu  | x86_64-pc-windows-msvc | aarch64-apple-darwin |
| aarch64-unknown-linux-gnu |                        |                      |
