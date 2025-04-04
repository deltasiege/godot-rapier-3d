# Identification

## GRUIDs

All rollback-aware objects are identified across the multiplayer network using `GRUID` _(Godot Rollback unique identifier)_

### Limits

GRUIDs are `(u8, u32)` (peer_index, object counter) which is 5 bytes of data, which means:

- up to 255 network peers can exist in one multiplayer game
- up to 4 billion rollback-aware objects per network peer can exist
- up to ~100 objects can be referred to in a single network packet (each peer sends their own packets)

In practical terms, this means your game can interact with (control, move or otherwise apply forces to) up to ~100 rollback-aware objects at once, per player, per physics frame. This does not include natural forces applied by the physics engine.

### Peer indexes

#### Peer index of 0

All objects that already exist when the game starts have their peer index set to 0. A peer index of 0 means that these objects were spawned by nobody and already exist for all peers. Therefore, they should never need to be sent over the network.

Any object spawned in a deterministic way by a peer_index 0 object should also have a peer_index of 0.

#### Other indices

| peer_index | meaning            |
| ---------- | ------------------ |
| 1          | server             |
| 2+         | other network peer |
