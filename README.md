# Connex

This is a game like the classic *Plumber Game*.  By rotating each small part, the whole is connected.

But the meaning of "connected" is different, unlike the plumber, in this game you can't have independent parts.

## Why create this

There are many crate in this workspace, the core part is `connex` and `connex-levels`.

`connex` defined the type and game logic, `connex-levels` is the level list.

Other crates, are just some kind of implements to show the gaming UI and dispatch user action to `connex` game logic to make this game playable.

The reason it's designed like this is that I want to use this game to learn all kinds of different game engines(bevy, Amethyst, Fyrox, Godot, etc.), GUI libraries(tui, egui, iced, durid), and even WASM and frontend UI framework.

This game has some level of complexity, implementing UI and event dispatching for it requires some understanding of the target framework. So, it will be something that I will try to port to the framework I'm learning after finish the "Hello World" of it, to consolidate my knowledge.

For now, there is only one implementation for `tui`, but it will become more in sometime.

## UI implementations

### Connex TUI

This is a implementation for playing connex in the terminal.

`cargo run -p connex-tui` to run it.

Gaming:

![connex tui game page][connex-tui-game-screenshot]

Help:

![connex tui help page][connex-tui-help-screenshot]

## LICENSE

BSD-3-Clause-Clear, See [LICENSE].

[connex-tui-game-screenshot]: https://rikka.7sdre.am/files/addcffb1-60ef-4f38-bcf1-e8d0020124a9.png
[connex-tui-help-screenshot]: https://rikka.7sdre.am/files/38ec9354-cfc9-4885-9d80-40091cb0d122.png
[LICENSE]: https://github.com/7sDream/connex/blob/master/LICENSE
