## Why am I building this?

- I want to make a coop multiplayer game:
  - using Godot
  - with "lagless" local controls (need rollback rather than lockstep)
  - that doesn't require dedicated servers (vulernable to hacking - competitive not advised)
  - that is in a large open world with physics objects (input-only determinism needed to avoid large network packets)

## What could it also be used for?

- Other multiplayer games that want to use rollback

## Why not use X?

- Spacetime DB
  - Doesnt have Godot support yet - would have to create 2nd class citizen rust module

## Random Notes

i want only a 1 or 2 frames of input delay locally for player character
(but make it configurable by player if they prefer artifacting - https://youtu.be/Y45rWIS3Qag?si=EKlinqvCuTAVp6Yg&t=388)

steam p2p - https://partner.steamgames.com/doc/features/multiplayer/networking
use a steam relay server so that there is time consistency

determinism - to avoid large network packets based on state of the world - only need to send inputs instead

all players must simulate the entire world because only inputs are exchanged
= world is constrained by player hardware not network

## reducing lag

https://mas-bandwidth.com/choosing-the-right-network-model-for-your-multiplayer-game/
"GGPO style deterministic lockstep" on this page

rollback - additional CPU cost, but with Rapier I think it might be ok? Also skipping all rendering and node creation during rollbacks should help shouldn't it?

buffer size of around 20 = 320 ms max lag
if ping goes higher:
relay server keeps progressing until it is <buffer length> frames ahead of the last received update from client peer, then that peer's inputs are dropped (ignored by the relay server once they finally come through). That peer is temporarily disconnected and must mid game re-join to recover

steam relay server: https://partner.steamgames.com/doc/features/multiplayer/steamdatagramrelay

if peer does not reconnect, they are kicked from the match. If that peer was the host, the host must be migrated.

rollback buffer size could be configured on each peer depending on hardware?
that would mean shitty hardware peers are less tolerant of lag

caveat - will probably need to use a few frames of input delay to help with artifacts

shit - not an option - alternative - no rollback, deterministic lockstep - simulation only progresses when all peers have inputs of all other peers
= one laggy player ruins experience

## mid-game rejoins

probably best to pause or slow down the game
connecting peer needs to download the world state at current tick and subsequent inputs being made
then fast forward through all inputs after state is downloaded
