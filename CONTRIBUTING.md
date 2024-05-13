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

## Development quickstart

1. Open this project in Godot
1. Make edits as desired
1. Run `cargo build` in the `/addons/godot-rapier-3d/rust` directory
1. The extension should automatically reload when Godot is refocused

## Bugs

Please raise an issue and provide reproducible steps or a minimal reproduction project, which is a small Godot project which reproduces the issue, with no unnecessary files included.

## Known issues

- Using any of the logging macros (`crate::error!` etc.) grabs a mutable reference to the engine singleton. If you already have done this in the same function, Godot will crash. You instead need to pass the engine bind:

  ```rust
  fn my_func() {
    let mut engine = crate::get_engine!();
    let mut bind = engine.bind_mut();
    // ...stuff...
    // crate::error!("This will crash Godot"); // INCORRECT
    crate::error!(bind; "This is fine");
  }
  ```

  the `get_engine!` macro should be upgraded to prevent such double access cases

## Roadmap

- [x] Visualize colliders
- [x] Snapshots & stepping
- [ ] Visualize active vs inactive bodies
- [ ] Full collider support
- [ ] Full rigid body support
- [ ] Determinism test bench
- [ ] Collision layers
- [ ] Save/load snapshots to/from resource files
- [ ] Editor UI to facilitate simulating in editor
- [ ] Gizmo handles for collider shapes
- [ ] Add to Godot asset library
