# bevy_wasm & bevy_wasm_sys

Mod your Bevy games with WebAssembly!

- [`bevy_wasm`](https://crates.io/crates/bevy_wasm) : For games
- [`bevy_wasm_sys`](https://crates.io/crates/bevy_wasm_sys) : For mods

See [examples/cubes](https://github.com/BrandonDyer64/bevy_wasm/tree/main/examples/cubes) for a comprehensive example of how to use this.

## Protocol

Our protocol crate defines the two message types for communicating between the game and mods.

```rust
use serde::{Deserialize, Serialize};

/// A message to be sent Mod -> Game.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ModMessage {
    Hello,
}

/// A message to be sent Game -> Mod.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum GameMessage {
    HiThere,
}
```

## Game

Our game will import `WasmPlugin` from [`bevy_wasm`](https://crates.io/crates/bevy_wasm), and use it to automatically send and receive messages with the mods.

```rust
use bevy::prelude::*;
use bevy_wasm::WasmPlugin;
use my_games_protocol::{GameMessage, ModMessage};

fn main() {
    let startup_mods = vec![
        include_bytes!("some_mod.wasm"),
        include_bytes!("some_other_mod.wasm"),
    ];

    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(WasmPlugin::<GameMessage, ModMessage>::new(startup_mods))
        .add_system(listen_for_mod_messages)
        .add_system(send_messages_to_mods)
        .run();
}

fn listen_for_mod_messages(mut events: EventReader<ModMessage>) {
    for event in events.iter() {
        match event {
            ModMessage::Hello => {
                println!("The mod said hello!");
            }
        }
    }
}

fn send_messages_to_mods(mut events: EventWriter<GameMessage>) {
    events.send(GameMessage::HiThere);
}
```

## Mod

Our mod will import `FFIPlugin` from [`bevy_wasm_sys`](https://crates.io/crates/bevy_wasm_sys), and use it to automatically send and receive messages with the game.

```rust
use bevy_wasm_sys::prelude::*;
use my_games_protocol::{GameMessage, ModMessage};

#[no_mangle]
pub unsafe extern "C" fn build_app() {
    App::new()
        .add_plugin(FFIPlugin::<GameMessage, ModMessage>::default())
        .add_system(listen_for_game_messages)
        .add_system(send_messages_to_game)
        .run();
}

fn listen_for_game_messages(mut events: EventReader<GameMessage>) {
    for event in events.iter() {
        match event {
            GameMessage::HiThere => {
                println!("The game said hi there!");
            }
        }
    }
}

fn send_messages_to_game(mut events: EventWriter<ModMessage>) {
    events.send(ModMessage::Hello);
}
```
