Ghastly is a work-in-progress alternative server for [GhostNet](https://gamebanana.com/gamefiles/6801),
a multiplayer mod for the 2018 game Celeste.
The reference server is slow, heavy-weight, and buggy.
Ghastly seeks to solve these problems through a clean rewrite
in the Rust programming language.
Rust is a programming language that is empowering everyone to build reliable and efficient software.

# Guide

(adapted from the [Everest wiki page](https://github.com/EverestAPI/Resources/wiki/How-do-I-play-Celeste-with-others-over-the-internet%3F-(GhostNet)/))

1. Download the [Everest API](https://everestapi.github.io/).
1. Download [GhostNet](https://gamebanana.com/gamefiles/6801).
1. If you didn't use 1-Click installation,
put the zip inside of your Mods folder in Celeste's data.
1. Load up Celeste after doing everything mentioned previously.
1. Go into Mod Settings, and down to GhostNet.
Set Name to whatever you want to be known as,
and set the server to `ghastly.leo60228.space`.
1. Switch Connected to On. If it works, congrats!

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
