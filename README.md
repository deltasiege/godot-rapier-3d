<p align="center">
<img src="assets/gr3d-logo.svg" width="64px"/>
</p>

# Godot Rapier 3D 🤺

<p align="center">
<img src="assets/gr3d-vid.gif"/>
</p>

## What is this?

A [GDExtension][gdext-link] that enables the [Rapier physics engine][rapier-link] within [Godot][godot-link].

It is _not_ a drop-in replacement for the Godot physics engine. Rapier nodes operate separately from Godot physics.

### Features

- Cross platform determinism
- Physics state manual stepping
- Physics state saving & loading

### Limitations

- No mobile support ([godot-rust](https://github.com/godot-rust/gdext/issues/24))

### Requirements

- Godot 4.2.2 or later

## Quickstart

1. Download or clone [addons/godot-rapier-3d](addons/godot-rapier-3d/) to the same directory within your Godot project - [download-directory.github.io/](https://download-directory.github.io/) is great for this
1. (temporary! downloadable release binaries coming soon) - Install rust and run `cargo build` in the `addons/godot-rapier-3d/rust` directory
1. Add some RapierRigidBody3D nodes to your scene and add a RapierCollider3D node as a child of each
1. Call `Rapier3D.step()` from within a `_physics_process()` function, or as often as you like

   ```gdscript
   func _physics_process(_delta):
     Rapier3D.step()
   ```

1. Run your game

Your physics objects should simulate! 🎉

Configure simulation and debug options by searching for `Rapier 3D` in your project settings

## Saving and loading state

Call `Rapier3D.get_state()` anywhere in your code to get a [`PackedByteArray`](https://docs.godotengine.org/en/stable/classes/class_packedbytearray.html) representing the current physics state.

Use `Rapier3D.set_state(snapshot)` to set the physics state to a snapshot.

Obtain a [hash](https://docs.godotengine.org/en/stable/classes/class_array.html#class-array-method-hash) of a snapshot using the `Rapier3D.get_hash(snapshot)` function.

```gdscript
var initial_snapshot

func _ready():
	Rapier3D.physics_ready.connect(_on_physics_ready)

func _on_physics_ready():
  initial_snapshot = Rapier3D.get_state()
  var hash = Rapier3D.get_hash(initial_snapshot)

func _on_foo():
  Rapier3D.set_state(initial_snapshot)
```

### Why `_on_physics_ready`?

Colliders need 1 extra frame to attach to physics bodies. Therefore it's recommended that you don't use `Rapier3D.get_state()` within a `_ready()` function because colliders will not be attached yet.

Instead, you should connect to the `Rapier3D.physics_ready` signal as shown above.

## Why does this exist?

Currently Godot does not support [on-demand physics simulation](https://github.com/godotengine/godot-proposals/issues/2821), does not have [built-in snapshotting](https://github.com/godotengine/godot-proposals/issues/7041), and is also not [deterministic](https://gafferongames.com/post/deterministic_lockstep).

These features are either important or required for creating networked games that use physics, depending on the chosen network architecture of your game.

Luckily, Godot 4 provides a great [extension system][gdext-link] and [Rapier][rapier-link] provides these missing features. 🚀

## Known issues

- `Project -> Reload current project` can cause the engine to not load properly - errors will print to the godot console. These errors can be safely ignored. To eliminate them, close godot entirely and reopen your project from the project manager.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)

## Attributions

- [dsnopek](https://github.com/dsnopek) and [SGPhysics2D](https://www.snopekgames.com/tutorial/2021/getting-started-sg-physics-2d-and-deterministic-physics-godot)
- [GameDevelopmentCenter](https://www.youtube.com/c/GameDevelopmentCenter)
- [appsinacup/godot-rapier-2](https://github.com/appsinacup/godot-rapier-2d)
- [ilyas-taouaou/rapier-gdext](https://github.com/ilyas-taouaou/rapier-gdext)

[rapier-link]: https://rapier.rs/
[godot-link]: https://godotengine.org/
[gdext-link]: https://docs.godotengine.org/en/stable/tutorials/scripting/gdextension/what_is_gdextension.html
