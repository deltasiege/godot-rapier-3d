# Flows

We MUST instance at the time of spawning in order to get transforms =<

## Spawning

1. GDScript `GR3D.spawn()`
2. Check ResourceCache if resource path has been mapped to Vec<NodeBlueprint> previously

If not:

1. Instantiate provided resource path (gives Gd<Node>)
2. Iterate and interrogate that instantiated tree to build a Vec<NodeBlueprint> tree. Collision shapes/areas as children of rigidbodies is supported. Nested rbs -> rbs or colliders -> colliders is not supported.
3. Add the resulting Vec<NodeBlueprint> to the ResourceCache
4. `queue_free` the interrogated tree - it will be recreated by Godot later if necessary

If it is, just use cached Vec<NodeBlueprint>

3. Push Spawn(Vec<NodeBlueprint>) into `awaiting_rapier` queue
4. During `GR3D.step()`, iterate + drain `awaiting_rapier` queue:

   - Iterate the Vec<NodeBlueprint> tree and insert Rapier objects into the Rapier world for each entry and collider children
   - Set resulting RapierHandles + blueprint on a new `NodeData` object
   - set the spawn_tick in the `NodeData` object to the current tick
   - insert `NodeData` object into node_db `nodes`
   - Add `Spawn(NodeData)` to `awaiting_godot` queue

5. During Godot physics process, iterate + drain `awaiting_godot` queue:
   - create Godot node from NodeData
   - set node_data property on the created node

## Despawning

## Ambiently enter_tree

1. Rust node `enter_tree`
2. Somehow construct NodeBlueprint

---

## FAQ

Why do we need GR3D.spawn? Cleaner if just listen to instantiate()? - A: No we don't always want to create the Godot node, so we need GR3D.spawn

# Spawning / despawning / ambient nodes and rolling back

For both GR3D.spawn + enter_tree (only for ambient peer_idx)

1. extract needed data from godot resource
2. (batch) create in rapier and get handle
3. push blueprint with spawn tick into node_db
4. Set GRUID action to `SPAWN` in `godot_action_list` (UNLESS ambient enter_tree! peer_idx 0)

configuration does not need to be queued or serialized! - its just part of rapier state, which is already rollback friendly! - godot will need to sync the node properties with rapier though after rolling back for read purposes

For GR3D.despawn

1. set end tick on node blueprint
2. (batch) delete the rapier object
3. queue_free the node
4. Set GRUID action to `DESPAWN` in `godot_action_list`

For exit_tree

1. Push error - use gr3d.despawn to delete rollback nodes!

On every physics frame

1. Godot needs to know which GRUIDs to create and which GRUIDs to destroy in order to match blueprint status

## Scenarios

### We receive a missed input from a remote peer which causes spawn

1. Blueprints with spawn ticks after beginning of rollback tick are destroyed and those GRUIDs added to `godot_action_list` as `DESPAWN`
2. Inputs are replayed
3. GR3D.spawn is replayed because of new input
4. New rapier object + blueprint is (batch) created
5. GRUID is added to `godot_action_list` as `SPAWN`
6. Rollback ends
7. Godot iterates all GRUIDs in `godot_action_list`
   - Retrieves blueprint from GRUID
   - Confirms node does not already exist at path in blueprint - error if it does
   - Retrieves current isometry from rapier handle in blueprint
   - Retrieves current property settings from rapier handle in blueprint
   - Create node in Godot, but suppress enter_tree event (avoid ambient spawn)

### We receive a missed input from a remote peer which causes despawn

1. Blueprints with spawn ticks after beginning of rollback tick are destroyed, and those GRUIDs added to `godot_action_list` as `DESPAWN`
2. Inputs are replayed
3. GR3D.despawn is replayed because of new input
4. Rapier object is (batch) deleted and end tick is recorded on blueprint
5. GRUID is added to `godot_action_list` as `DESPAWN`
6. Rollback ends
7. Godot iterates all GRUIDs in `godot_action_list`

   - Retrieves blueprint from GRUID
   - Confirms node still exists at path in blueprint - error if it doesn't
   - `queue_free` the node

## Cleaning up blueprints

Every 10k ticks, iterate all blueprints and clean up blueprints that have been despawned for a long time, and cant possible be spawned during rollback
