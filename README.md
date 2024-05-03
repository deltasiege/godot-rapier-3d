# Godot Rapier 3D

## Notes

In Cargo.toml this block makes rust code compile slower, but rapier more optimized and efficient

```toml
[profile.dev.package.rapier3d]
opt-level = 3
```

- https://www.rapier.rs/docs/user_guides/rust/common_mistakes#my-local-build-of-rapier-is-slower-than-the-online-demos
- https://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/book/second-edition/ch14-01-release-profiles.html#:~:text=The%20opt%2Dlevel%20setting%20controls,the%20resulting%20code%20running%20slower.

## Credits

Inspired by [ilyas-taouaou/rapier-gdext](https://github.com/ilyas-taouaou/rapier-gdext)
