# Godot Rapier 3D

## Roadmap

- [] Editor gizmos for collider shapes
- [] Serialize/deserialize physics state to/from variable
- [] Save/load physics state to/from resource file

## Contributing

See [CONTRIBUTING.md]()

## Credits

Inspired by [ilyas-taouaou/rapier-gdext](https://github.com/ilyas-taouaou/rapier-gdext)

## TODO

1. collision shapes as resource
1. Simple example
1. On init, register all found rigid bodies and colliders
1. Maybe wait 3 frames before step starts working to give colliders a chance to connect? (call deferred) (document this in readme)
1. refactor utils engine singleton function to include pipeline retrieval

## Saving/loading

- https://rapier.rs/docs/user_guides/rust/serialization

1. physics state struct
1. make step(true) return the serialized physics state
1. godot is responsible for saving the serialized state into memory or a resource file
1. provide helper functions in Autoload to facilitate both cases

1. singleton should provide load() function that takes serialized state
1. overwrite all values in the PhysicsState struct with the loaded ones
1. continue as normal
