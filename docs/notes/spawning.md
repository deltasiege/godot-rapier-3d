# Rollback Spawn Manager

Handles spawning + despawning all rollback-aware nodes

Reference: https://gitlab.com/BimDav/delta-rollback/-/blob/e8514be34dfe01bc505579cda7df66a2faae51d4/addons/delta_rollback/SpawnManager.gd

Whenever a node is spawned, we must record:

- GRUID
- node name
- parent path
- resource path of the packed scene that was spawned

## Receiving remote actions flows

### We deleted node referred to by remote action

If a remote action has a GRUID that refers to a local peer_index node in our Lookup Table that we have since despawned locally, the smart pointer will be invalid at the current tick.

Assuming receiving the remote action caused a rollback, the following happens:

We consult our local spawn buffer and spawn any

, it just needs to recreate the node that it spawned previously.Then, it needs to suppress that node's `on_enter_tree` event from adding more actions to the local_buffer.

The node will be configured the same way in Rapier automatically it was when it was originally spawned because the settings on the Godot node should match, which is what the `ConfigureNode` action reads from

After re-creating the node, `RollbackMultiplayerSpawner` must update this Lookup Table with the new Gd smart pointer so that actions referencing it can be processed

Later when the `AddNode` action is processed, this lookup table is updated again to update the Rapier handle to the newly added one.

### A remote action has a GRUID that is not in our local Lookup Table

That means the remote peer has spawned an object that we haven't spawned yet.

## Spawning players

Server locally spawns a player via `GR3D.spawn()`
---> `GR3D.spawn()` adds a SpawnEvent into local buffer
------> SpawnEvent includes all necessary data to recreate that node on other peers

---> `GR3D.spawn()` sends spawn event to all peers
------> All peers add the spawn event into their own spawn buffer
------> All peers rollback and re-create that node from the spawn event

Actions refer to Rapier actions only (current)

## Destroying peer_index 0 objects

All peers already have peer_index 0 objects

Non-server peer A destroys a peer_index 0 object via `GR3D.despawn()`
---> `GR3D.despawn()` adds a DespawnEvent into local buffer

---

When rolling back, we don't want to recreate Godot nodes,
we only want to recreate them if they are meant to be present at the end of rollback but they are missing

But how do the actions grab their needed data? Transform / shape type etc.

!! Rapier configuration needs to be separate from Godot Nodes !! Actions should retrieve from SpawnEvent

It should be called `NodeRegistry`
