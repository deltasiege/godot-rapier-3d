# Godot Rapier 3D

## Contributing

See [CONTRIBUTING.md]()

## Credits

Inspired by [ilyas-taouaou/rapier-gdext](https://github.com/ilyas-taouaou/rapier-gdext)

## TODO

1. Visualise colliders
1. On init, register all found rigid bodies and colliders
1. Save/load state
1. Switch to editor singleton for guaranteed access to scene tree?
1. Maybe wait 3 frames before step starts working to give colliders a chance to connect? (call deferred)
1. refactor engine singleton function to include pipeline retrieval

## Saving/loading

- https://rapier.rs/docs/user_guides/rust/serialization

1. physics state struct
1. make step(true) return the serialized physics state
1. godot is responsible for saving the serialized state into memory or a resource file
1. provide helper functions in Autoload to facilitate both cases

1. singleton should provide load() function that takes serialized state
1. overwrite all values in the PhysicsState struct with the loaded ones
1. continue as normal
