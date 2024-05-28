# Godot Rapier 3D 🤺

<p align="center">
<img src="assets/gr3d-vid.gif"/>
</p>

## What is this?

A [GDExtension][gdext-link] that enables the [Rapier physics engine][rapier-link] within [Godot][godot-link].

It is _not_ a drop-in replacement for the Godot physics engine. Rapier nodes operate separately from Godot physics.

### Features

- Cross platform determinism ✔️ (confirmed via [actions](https://github.com/deltasiege/godot-rapier-3d/actions/workflows/build-and-test.yml)!)
- Physics state manual stepping ✔️
- Physics state saving & loading ✔️

### Requirements

- Godot 4.2.2 or later

## Quickstart

1. Download the [latest --all release](https://github.com/deltasiege/godot-rapier-3d/releases/latest)  
1. Extract the release archive into your godot project's root directory
1. Add RapierRigidBody3D nodes to your scene and some RapierCollider3D + MeshInstance3D nodes as children of the rigid bodies
1. Run your game

Your physics objects should simulate! 🎉

## Configuring

By default, the `Rapier3DDebugger` autoload will start the physics simulation for you. To get more control over when you simulate, search for `Rapier 3D` in your project settings and disable either `Debug in Game` or `Show UI` under the Debug category.

Now you can call `Rapier3D.step()` from within any `_physics_process()` function, or as often as you like. This function advances the physics simulation by 1 step.

```gdscript
func _physics_process(_delta):
  Rapier3D.step()
```

## Saving and loading state

Call `Rapier3D.get_state()` anywhere in your code to get a [`PackedByteArray`](https://docs.godotengine.org/en/stable/classes/class_packedbytearray.html) representing the current physics state.

Use `Rapier3D.set_state(snapshot)` to set the physics state to a snapshot.

Obtain a [hash](https://docs.godotengine.org/en/stable/classes/class_array.html#class-array-method-hash) of a snapshot using the `Rapier3D.get_hash(snapshot)` function.

```gdscript
var initial_snapshot

func _ready():
  initial_snapshot = Rapier3D.get_state()
  var hash = Rapier3D.get_hash(initial_snapshot)

func _on_foo():
  Rapier3D.set_state(initial_snapshot)
```

## Why does this exist?

Currently Godot does not support [on-demand physics simulation](https://github.com/godotengine/godot-proposals/issues/2821), does not have [built-in snapshotting](https://github.com/godotengine/godot-proposals/issues/7041), and is also not [deterministic](https://gafferongames.com/post/deterministic_lockstep).

These features are either important or required for creating networked games that use physics, depending on the chosen network architecture of your game.

Luckily, Godot 4 provides a great [extension system][gdext-link] and [Rapier][rapier-link] provides these missing features. 🚀

## Limitations

- No mobile support ([godot-rust](https://github.com/godot-rust/gdext/issues/24))

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

Thanks very much for your help

[rapier-link]: https://rapier.rs/
[godot-link]: https://godotengine.org/
[gdext-link]: https://docs.godotengine.org/en/stable/tutorials/scripting/gdextension/what_is_gdextension.html
