## Requirements

- Godot 4.4 or later

## Beware of limitations

- Web & mobile builds are not yet stable ([godot-rust](https://github.com/godot-rust/gdext/issues/24))
- `rv64`, `ppc64` and `ppc32` architectures are not yet confirmed for determinism via automated testing because they are [not yet officially supported by Godot](https://github.com/godotengine/godot-proposals/issues/3374#issuecomment-2142165372)

## Let's go

### Installation

1. Download the [latest --all release](https://github.com/deltasiege/godot-rapier-3d/releases/latest)
1. Extract the release archive into your godot project's root directory

### Add a rigid body

1.  Add a RapierRigidBody3D node to your scene
1.  Add 3 child nodes as children of the rigid body:

    - RapierCollisionShape3D
    - MeshInstance3D
    - CollisionShape3D

1.  Assign the CollisionShape3D as the `shape` field on the RapierCollisionShape3D node
1.  Enable `Debug` -> `Visible Collision Shapes`
1.  Run your game

Your physics objects should simulate! 🎉
