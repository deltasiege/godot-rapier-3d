# Godot Rapier 3D

## Requirements

- Godot 4.2.2 or later

## Roadmap

- [x] Visualize colliders
- [x] Serialize/deserialize physics state to/from variable
- [ ] Save/load physics state to/from resource file
- [ ] Interface for stepping in editor
- [ ] Gizmos handles for collider shapes
- [ ] Collision layers

## Contributing

See [CONTRIBUTING.md]()

## References

- [ilyas-taouaou/rapier-gdext](https://github.com/ilyas-taouaou/rapier-gdext)

## TODO

1. switching scenes leaves orphan colliders
1. collision shapes as resource - issues
1. On editor init, register all found rigid bodies and colliders
1. Maybe wait 3 frames before step starts working to give colliders a chance to connect? (call deferred) (document this in readme)
