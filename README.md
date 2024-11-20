# Bevy WASM

Mod your Bevy games with WebAssembly!

[![CI](https://github.com/BrandonDyer64/bevy_wasm/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/BrandonDyer64/bevy_wasm/actions)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/BrandonDyer64/bevy_wasm#license)
[![Crates.io](https://img.shields.io/crates/d/bevy_wasm.svg?color=blue)](https://crates.io/crates/bevy_wasm)<br/>
[![Bevy](https://img.shields.io/badge/bevy-v0.10-blueviolet)](https://crates.io/crates/bevy)

|                    |                                                                                                                                                                                            |               |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------- |
| `bevy_wasm`        | [![](https://img.shields.io/crates/v/bevy_wasm.svg)](https://crates.io/crates/bevy_wasm) [![](https://docs.rs/bevy_wasm/badge.svg)](https://docs.rs/bevy_wasm)                             | For games     |
| `bevy_wasm_sys`    | [![](https://img.shields.io/crates/v/bevy_wasm_sys.svg)](https://crates.io/crates/bevy_wasm_sys) [![](https://docs.rs/bevy_wasm_sys/badge.svg)](https://docs.rs/bevy_wasm_sys)             | For mods      |
| `bevy_wasm_shared` | [![](https://img.shields.io/crates/v/bevy_wasm_shared.svg)](https://crates.io/crates/bevy_wasm_shared) [![](https://docs.rs/bevy_wasm_shared/badge.svg)](https://docs.rs/bevy_wasm_shared) | For protocols |

See [examples/cubes](https://github.com/BrandonDyer64/bevy_wasm/tree/main/examples/cubes) for a comprehensive example of how to use this.

[Changelog](https://github.com/BrandonDyer64/bevy_wasm/blob/main/CHANGELOG.md)

## Examples

To start the examples, you will need those dependencies

```shell
rustup target add wasm32-unknown-unknown
rustup target add wasm32-wasip1
cargo install cargo-make
```

Native executable calling a wasm32-wasip1 plugin
```shell
cargo make run-simple
cargo make run-cubes
cargo make run-shared-resources
```

## Protocol

Our protocol crate defines the two message types for communicating between the game and mods.

```toml
[dependencies]
bevy_wasm_shared = "0.10"
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
bevy = "0.10"
bevy_wasm = "0.10"
my_game_protocol = { git = "https://github.com/username/my_game_protocol" }
```

```rust
use bevy::prelude::*;
use bevy_wasm::prelude::*;
use my_game_protocol::{GameMessage, ModMessage, PROTOCOL_VERSION};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WasmPlugin::<GameMessage, ModMessage>::new(PROTOCOL_VERSION))
        .add_startup_system(add_mods)
        .add_system(listen_for_mod_messages)
        .add_system(send_messages_to_mods)
        .run();
}

fn add_mods(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(WasmMod {
        wasm: asset_server.load("some_mod.wasm"),
    });
    commands.spawn(WasmMod {
        wasm: asset_server.load("some_other_mod.wasm"),
    })
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
bevy_wasm_sys = "0.10"
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

## Sharing Resources

**Protocol:**

```rust
#[derive(Resource, Default, Serialize, Deserialize)]
pub struct MyResource {
    pub value: i32,
}
```

**Game:**

```rust
App::new()
    ...
    .add_resource(MyResource { value: 0 })
    .add_plugin(
        WasmPlugin::<GameMessage, ModMessage>::new(PROTOCOL_VERSION)
            .share_resource::<MyResource>()
    )
    .add_system(change_resource_value)
    ...

fn change_resource_value(mut resource: ResMut<MyResource>) {
    resource.value += 1;
}
```

**Mod:**

```rust
App::new()
    ...
    .add_plugin(FFIPlugin::<GameMessage, ModMessage>::new(PROTOCOL_VERSION))
    .add_startup_system(setup)
    .add_system(print_resource_value)
    ...

fn setup(mut extern_resource: ResMut<ExternResources>) {
    extern_resources.insert::<MyResource>();
}

fn print_resource_value(resource: ExternRes<MyResource>) {
    println!("MyResource value: {}", resource.value);
}
```

See [examples/shared_resources](https://github.com/BrandonDyer64/bevy_wasm/tree/main/examples/shared_resources) for a full example.

## Roadmap

|     |                                                  |
| --- | ------------------------------------------------ |
| ✅  | wasmtime runtime in games                        |
| ✅  | Send messages from mods to game                  |
| ✅  | Send messages from game to mods                  |
| ✅  | Multi-mod support                                |
| ✅  | Time keeping                                     |
| ✅  | Protocol version checking                        |
| ✅  | Extern Resource                                  |
| ✅  | Startup system mod loading                       |
| ✅  | Direct update control                            |
| ✅  | Mod unloading                                    |
| ✅  | Mod discrimination (events aren't broadcast all) |
| ✅  | Browser support                                  |
| ⬜  | Extern Query                                     |
| ⬜  | Synced time                                      |
| ⬜  | Mod hotloading                                   |
| ⬜  | Automatic component syncing                      |

## License

Bevy WASM is free, open source and permissively licensed!
Except where noted (below and/or in individual files), all code in this repository is dual-licensed under either:

-   MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
-   Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.
This means you can select the license you prefer!
This dual-licensing approach is the de-facto standard in the Rust ecosystem and there are [very good reasons](https://github.com/bevyengine/bevy/issues/2373) to include both.

### Your contributions

Unless you explicitly state otherwise,
any contribution intentionally submitted for inclusion in the work by you,
as defined in the Apache-2.0 license,
shall be dual licensed as above,
without any additional terms or conditions.
