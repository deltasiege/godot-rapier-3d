# Godot Rapier 3D 🤺

## What is this?

A [GDExtension][gdext-link] that enables the [Rapier physics engine][rapier-link] within [Godot][godot-link].

It is _not_ a drop-in replacement for the Godot physics engine. Rapier nodes operate separately from Godot physics.

### Features

- Cross platform determinism ✔️ (confirmed via [actions](https://github.com/deltasiege/godot-rapier-3d/actions/workflows/build-and-test.yml)!)
- Physics manual stepping ✔️
- Physics saving & loading ✔️

### Requirements

- Godot 4.4 or later

## Quickstart

1. Download the [latest --all release](https://github.com/deltasiege/godot-rapier-3d/releases/latest)
1. Extract the release archive into your godot project's root directory
1. Add a RapierRigidBody3D node to your scene and RapierCollisionShape3D + MeshInstance3D nodes as children of the rigid body
1. Run your game

Your physics objects should simulate! 🎉

## Roadmap

This extension is currently under heavy development, compatibility when upgrading versions is not assured until 1.0.0

- [x] Visualize colliders
- [x] Snapshots & stepping
- [x] Determinism automated testing
- [x] Collider shapes
- [ ] Forces and manipulating rigidbodies
- [ ] Visualize active vs inactive bodies
- [ ] Collision layers
- [ ] Save/load snapshots to/from resource files
- [ ] Editor UI to facilitate simulating in editor
- [ ] Gizmo handles for collider shapes
- [ ] Add to Godot asset library
- [ ] [Property comments](https://github.com/godot-rust/gdext/issues/178)

## Limitations

- No mobile support ([godot-rust](https://github.com/godot-rust/gdext/issues/24))
- `rv64`, `ppc64` and `ppc32` architecture automated testing for determinism [not yet officially supported](https://github.com/godotengine/godot-proposals/issues/3374#issuecomment-2142165372)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)

## Attributions

- [gdext](https://github.com/godot-rust/gdext) discord community
  - [Lili Zoey](https://github.com/lilizoey)
  - [Bromeon](https://github.com/Bromeon)
- [dsnopek](https://github.com/dsnopek) and [SGPhysics2D](https://www.snopekgames.com/tutorial/2021/getting-started-sg-physics-2d-and-deterministic-physics-godot)
- [GameDevelopmentCenter](https://www.youtube.com/c/GameDevelopmentCenter)
- [appsinacup/godot-rapier-2](https://github.com/appsinacup/godot-rapier-2d)
- [ilyas-taouaou/rapier-gdext](https://github.com/ilyas-taouaou/rapier-gdext)

[rapier-link]: https://rapier.rs/
[godot-link]: https://godotengine.org/
[gdext-link]: https://docs.godotengine.org/en/stable/tutorials/scripting/gdextension/what_is_gdextension.html
