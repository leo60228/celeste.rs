Ghastly is a work-in-progress alternative server for [GhostNet](https://gamebanana.com/gamefiles/6801),
a multiplayer mod for the 2018 game Celeste.
The reference server is slow, heavy-weight, and buggy.
Ghastly seeks to solve these problems through a clean rewrite
in the Rust programming language.
Rust is a programming language that is empowering everyone to build reliable and efficient software.

# Features

Ghastly is not yet complete, and currently implements the following features:

* Chat
* Player listings
* Updates
  * Partial: Updates are currently sent to all players,
  when it would be preferable to only send them to players in the same room.
  However, as updates are sent over UDP, this should not cause network congestion.

## WIP

The following features are currently a work in progress:

* Player collision is currently parsed, but not acted upon.

## Note

Support for sending updates over TCP is intentionally not supported.
It was a stopgap measure to improve behavior on networks without UDP support,
however the implementation caused network congestion.

Some networks support receiving UDP but not sending it,
which is still supported by this server.
