# Node cache

!!! I can avoid cache altogether if I only send rapier handle over the network = 8 bytes !!!
might still be nice to reduce it down to 4 bytes - but can leave that as nice to have

reduce cuid length to save bytes down to 10 characters

node_cache must work like this: (cuids should be smaller than paths)

Peer A sends action with rapier handle + cache index several times until acknowledged
Peer B applies action and caches idx -> CUID
Peer B replies with acknowledgement that node path has been cached against CUID
Peer A uses cache index to refer to that node from now on

node path should only be needed when dealing with godot stuff, so we shouldn't need to send it over the network
if spawners are already in the node tree, node path will always be known to both sides without sending it over the network

--
cuid can be converted to node path via lookup table
