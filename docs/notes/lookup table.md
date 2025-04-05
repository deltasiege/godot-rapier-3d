# Lookup Table

Should provide functions:

- GRUID + tick -> Rapier handle
- GRUID + tick -> Gd smart pointer
- Rapier handle + tick -> GRUID
- Rapier handle + tick -> Gd smart pointer

Stores all references for all ticks

Gd smart pointers only need to be fetched once via get_node_or_null
and then every tick should return that same pointer unless the node is reparented

Needs to be updated whenever node is reparented

## Data

RollbackNodeReference = { GRUID, Option<rapier handle>, Option<Gd smart pointer> }

GRUID map = HashMap<GRUID, HashMap<tick, RollbackNodeReference>>
Rapier map = HashMap<Rapier handle, HashMap<tick, RollbackNodeReference>>

## Examples

GRUID A, tick 10 = /abc
GRUID A, tick 20 = /abc/123

get GRUID A @ tick 15 = get highest tick value below given (15) = 10 = /abc
get GRUID A @ tick 5 = invalid, doesn't exist yet
get GRUID A @ tick 20 = /abc/123
get GRUID A @ tick 2000 = /abc/123

## Flows

Entry should be added to Lookup Table whenever a node enters the tree (game start + when a node is spawned). Gd smart pointer is definitely known at this point. Rapier handle is not known yet, will be added when `AddNode` action is processed.

Entry is also added whenever a node exits the tree (queue_free'd). Gd smart pointer should be set to `None` immediately. Rapier handle will later be set to None when `RemoveNode` action is processed - Whenever the corresponding Rapier object is added / removed within Rapier, this lookup table is updated with the correct rapier handle

When an action is received from a remote peer, we only know the GRUID, not the smart pointer, lookup table is accessed to retrieve the smart pointer at the tick that the action refers to.
