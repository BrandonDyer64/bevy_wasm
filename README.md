# Bevy WASM

Mod your Bevy games with WebAssembly!

|                    |                                                                                                                                                                                            |               |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------- |
| `bevy_wasm`        | [![](https://img.shields.io/crates/v/bevy_wasm.svg)](https://crates.io/crates/bevy_wasm) [![](https://docs.rs/bevy_wasm/badge.svg)](https://docs.rs/bevy_wasm)                             | For games     |
| `bevy_wasm_sys`    | [![](https://img.shields.io/crates/v/bevy_wasm_sys.svg)](https://crates.io/crates/bevy_wasm_sys) [![](https://docs.rs/bevy_wasm_sys/badge.svg)](https://docs.rs/bevy_wasm_sys)             | For mods      |
| `bevy_wasm_shared` | [![](https://img.shields.io/crates/v/bevy_wasm_shared.svg)](https://crates.io/crates/bevy_wasm_shared) [![](https://docs.rs/bevy_wasm_shared/badge.svg)](https://docs.rs/bevy_wasm_shared) | For protocols |

See [examples/cubes](https://github.com/BrandonDyer64/bevy_wasm/tree/main/examples/cubes) for a comprehensive example of how to use this.

## Protocol

Our protocol crate defines the two message types for communicating between the game and mods.

```toml
[dependencies]
bevy_wasm_shared = "0.9"
serde = { version = "1.0", features = ["derive"] }
```

```rust
use bevy_wasm_shared::prelude::*;
use serde::{Deserialize, Serialize};

/// The version of the protocol. Automatically set from the `CARGO_PKG_XXX` environment variables.
pub const PROTOCOL_VERSION: Version = version!();

/// A message to be sent Mod -> Game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModMessage {
    Hello,
}

/// A message to be sent Game -> Mod.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameMessage {
    HiThere,
}
```

## Game

Our game will import `WasmPlugin` from [`bevy_wasm`](https://crates.io/crates/bevy_wasm), and use it to automatically send and receive messages with the mods.

```toml
[dependencies]
bevy = "0.9"
bevy_wasm = "0.9"
my_game_protocol = { git = "https://github.com/username/my_game_protocol" }
```

```rust
use bevy::prelude::*;
use bevy_wasm::prelude::*;
use my_game_protocol::{GameMessage, ModMessage, PROTOCOL_VERSION};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(
            WasmPlugin::<GameMessage, ModMessage>::new(PROTOCOL_VERSION)
                .with_mod(include_bytes!("some_mod.wasm"))
                .with_mods(include_bytes!("some_other_mod.wasm"))
        )
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

```toml
[dependencies]
bevy_wasm_sys = "0.9"
my_game_protocol = { git = "https://github.com/username/my_game_protocol" }
```

```rust
use bevy_wasm_sys::prelude::*;
use my_game_protocol::{GameMessage, ModMessage, PROTOCOL_VERSION};

#[no_mangle]
pub unsafe extern "C" fn build_app() {
    App::new()
        .add_plugin(FFIPlugin::<GameMessage, ModMessage>::new(PROTOCOL_VERSION))
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

## Roadmap

|     |                                                  |
| --- | ------------------------------------------------ |
| ✅  | wasmtime runtime in games                        |
| ✅  | Send messages from mods to game                  |
| ✅  | Send messages from game to mods                  |
| ✅  | Multi-mod support                                |
| ✅  | Time keeping                                     |
| ✅  | Protocol version checking                        |
| ⬜  | Startup system mod loading                       |
| ⬜  | Custom FFI                                       |
| ⬜  | Mod discrimination (events aren't broadcast all) |
| ⬜  | `AssetServer` support and `Handle<WasmMod>`      |
| ⬜  | Mod unloading                                    |
| ⬜  | Direct update control                            |
| ⬜  | Mod hotloading                                   |
| ⬜  | Automatic component syncing                      |
| ⬜  | Browser support                                  |
