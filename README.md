# Godot Rapier 3D

## Requirements

- Godot 4.2.2 or later

## Roadmap

- [x] Visualize colliders
- [ ] Serialize/deserialize physics state to/from variable
- [ ] Save/load physics state to/from resource file
- [ ] Interface for stepping in editor
- [ ] Gizmos handles for collider shapes
- [ ] Collision layers

## Contributing

See [CONTRIBUTING.md]()

## References

- [ilyas-taouaou/rapier-gdext](https://github.com/ilyas-taouaou/rapier-gdext)

## TODO

1. collision shapes as resource - issues
1. On editor init, register all found rigid bodies and colliders
1. Maybe wait 3 frames before step starts working to give colliders a chance to connect? (call deferred) (document this in readme)

## Saving/loading

- https://rapier.rs/docs/user_guides/rust/serialization

1. physics state struct
1. make step(true) return the serialized physics state
1. godot is responsible for saving the serialized state into memory or a resource file
1. provide helper functions in Autoload to facilitate both cases

1. singleton should provide load() function that takes serialized state
1. overwrite all values in the PhysicsState struct with the loaded ones
1. continue as normal

## Visualizing colliders

1. add Rapier3DDebugger as an autoload (which is a tool script) (both editor and runtime version can be in @tool)
1. Rapier3DDebugger calls RapierDebugRenderPipeline render method in \_process loop
1. render method needs to call singleton to get the physics state
1. RapierDebugRenderPipeline wants draw line
1. call back up to Rapier3DDebugger maybe??
1. use same techniques as debug_draw to draw lines

call autoloads - could be useful for log func
