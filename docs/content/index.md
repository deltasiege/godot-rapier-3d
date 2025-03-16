# Welcome

## What is this?

A [GDExtension][gdext-link] that enables the [Rapier physics engine][rapier-link] within [Godot][godot-link].

It is _not_ a drop-in replacement for the Godot physics engine. Rapier nodes operate separately from Godot physics.

## Why does this exist?

_TLDR_ - For creating rollback based multiplayer games in Godot

Godot is currently missing these features:

- physics snapshotting
- on-demand physics simulation
- cross-platform physics determinism

Each of these features are required to implement a [rollback](https://en.wikipedia.org/wiki/Netcode#Rollback) networking system in a multiplayer game that uses physics.

Luckily, Godot 4 provides a great extension system that allows us to integrate Rapier, which is a physics engine that has these features 🚀

## Where to next?

- [Quickstart](./quickstart.md)
- [Manual stepping](./stepping.md)
- [Snapshotting](./snapshotting.md)
- [Is GR3D actually deterministic? Prove it!](./determinism.md)

[rapier-link]: https://rapier.rs/
[godot-link]: https://godotengine.org/
[gdext-link]: https://docs.godotengine.org/en/stable/tutorials/scripting/gdextension/what_is_gdextension.html
