## TODO

- Collision shapes will need to read transform from col_shape field rather than their own
- Kinematic align to slopes
- Determinism test
- Homogenize character scripts

- PID character needs
  - max steepness

## Notes

- Beware unsupported usage of `base.to_gd()` in `/rust/src/nodes/*` init functions https://github.com/godot-rust/gdext/issues/557
- Custom base classes are unfortunately not supported by `godot-rust` - would reduce a lot of code duplication
