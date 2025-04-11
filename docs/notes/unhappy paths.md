send latest sync frame all peers -> all peers in updates
send mismatches all peers -> all peers

if host sees that a peer has mismatches, it triggers fatal event

on fatal event

- game is paused
- show pings of all peers
- all peers can vote on:
  - kick the lagging peer
  - let peer download current state from the host and try to continue
