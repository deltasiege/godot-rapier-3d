# Implementation flow

## Game start

- GR3DRuntime

  - if we are not the server, return
  - inspects all objects that already exist in the game, sets their peer_index to 0, and ensures no conflicts
  - tell each peer what their peer index is, so that they can prefix their GRUID with it

- All rollback objects
  - Spawn a child node whose name is their GRUID. This is useful for finding the object by its GRUID

## Server spawns all players

- User Godot code

  - spawns players via MultiplayerSpawner node:
    - sets GRUID of each spawned player, prefixed with `1` (meaning they were spawned by the server)
    - sets the multiplayer_authority on each player (matches that player's peer_id)
    - replicates all players to all peers

- IRollbackObject on server + all peers
  - Store the spawned GRUID against the NodePath in a LookupTable (will match across all peers). This is used when deserializing LeanActions from other peers

## Non-server peer wants to move their player

- User Godot code

  - calls `move_by_amount` or similar GR3D singleton function

- GR3D singleton

  - adds `MoveNode` action to local + combined world_buffer
  - applies actions to world
  - steps world
  - player is moved locally

- GR3DNet singleton on local peer

  - compresses local world_buffer actions into LeanActions
  - serializes and sends LeanActions to all peers

- GR3DNet singleton on remote peers
  - deserializes and converts LeanActions into Actions
  - inserts

## Non-server peer wants to spawn a projectile

- Non-server peer
  - Adds an `AddNode` action to its local buffer, uses its next GRUID provided by the server
  -
