# Decision Record

All codebase decisions are recorded here so that they can be referred to/challenged later

## What to send over the network

Only inputs are sent over the network.

- Smaller packets
- Harder to exploit by hackers without requiring a server. Each peer's game logic should automatically prevent spoofed inputs producing a bad experience (i.e. pressing forward 200 times in a single frame still only moves forward once on that frame for other peers).

I sometimes fall into a trap of thinking I need to send state over the network when spawning something. Any time you think you need to send state over the network, think: "Why can't this synced state be achieved by all peers starting from the same point and then executing logic in the same way?"

## P2P versus client/server

We are going to always designate the host as the state authority because:

1. In both architectures, we are going to send inputs from all peers to all peers. Nothing is routed through server first so there is no network performance penalty either way.

2. Designating a state authority allows the following functions:
   - If only a minority of clients have a state mismatch, those clients can be kicked/re-synced instead of killing the game entirely
   - Trusted mid-game joins are possible by receiving the authoritative state from the server

The host can either be a regular player hosting a lobby, or it could be a dedicated server in the cloud. Since the host is trusted to have accurate state (and can exploit that if they wanted to), players should not be able to join lobbies hosted by other players that they dont know/trust - else they risk that host sending them a hacked game state during mismatch recovery or mid-game join.

## When are node blueprints created?

- When GR3D.spawn is called
- When a node enter_tree is called, if not already defined by GR3D.spawn

Ambient editor nodes will not have a blueprint until game runtime starts and they enter_tree.
Spawned projectiles etc. will immediately have a blueprint via GR3D.spawn.

We do it this way so that actual spawning of the godot nodes can be skipped during rollbacks (via GR3D.spawn).
