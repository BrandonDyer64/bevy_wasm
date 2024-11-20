# Cubes Example

This example shows how to make a somewhat more complex game with multiple mods.

One of the mods uses [Bevy](https://bevyengine.org/), while the other does not.

Note that the wasm size of `mod_without_bevy` is much smaller than `mod_with_bevy`.
This is, in part, due to `bevy_app` importing `wasm-bindgen` when targeting `wasm32-unknown-unknown`.

Folders here:

- `cubes` - The main game.
- `cubes_protocol` - Message structures for communicating between the game and mods.
- `mod_with_bevy` - A mod for our cubes game that uses Bevy.
- `mod_without_bevy` - A mod for our cubes game that does not use Bevy.

## Running Example

To run this example, use `cargo make run-cubes` from the project root.

You should see two cubes.
One green, moving up and down, is from our mod that is using Bevy.
One red, moving left and right, is from our mod that is _NOT_ using Bevy.

## About

A typical game shipped using `bevy_wasm` is expected to be split into two repositories:

- The game repository, which contains the game code. (Closed source)
- The protocol repository, which contains the message structures. (Open source)

Of course, your game doesn't need to be closed source, :), but the only part that needs to be open source is the protocol library.

It should also be common practice to include `publish = false` in the `Cargo.toml` of the protocol library, so that it is not published to crates.io.

Mod developers should import your protocol library from git, like so:

```toml
[dependencies]
somecoolgame_protocol = { git = "https://github.com/your-username/somecoolgame_protocol" }
bevy_wasm_sys = "0.1"
```
