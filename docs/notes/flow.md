# Implementation flow

# Setup

1. Client A+B+C all launch the game
2. Client A hosts a lobby
3. B+C connect to that lobby
4. Host A sends start game message and waits short time for B+C to sync
   Start game message informs each client what their peer_index is.
   0 = ambient, 1 = host, 2+ = client

5. ABC start their games in sync

# In game

1. All peers register all ambient Rollback objects
   (which already have GRUIDs assigned in their metadata) into their NodeDatabase
   and ensure no conflicts

1. All peers register their input adapters automatically

1. All peers send their inputs to all other peers every physics frame

1. All peers spawn their own player object + all other peer player objects via
   GR3D.spawn() in GDScript
   Each player object has its authority set according to peer_index and peer_id

# Client B moves their player

1. Client B pushes W to move their player forward on tick X

2. GDScript on Client B's character checks
   GR3D.get_input("input_key", self)

3. GR3D.get_input checks the multiplayer authority of the given `self` node,
   - if local, returns `input_adapter`.get_input which is immediate
   - if remote, returns the buffered remote peer input for the current tick
