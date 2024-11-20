//! Add this plugin to your Bevy app to enable WASM-based modding

use bevy::prelude::*;
use bevy_wasm_shared::prelude::*;
use colored::*;

use crate::{
    runtime::WasmRuntime,
    systems::{self, load_instances},
    wasm_asset::{WasmAsset, WasmAssetLoader},
    Message, SharedResource,
};

trait AddSystemToApp: Send + Sync + 'static {
    fn add_system_to_app(&self, app: &mut App);
}

struct ResourceUpdater<R: SharedResource> {
    _r: std::marker::PhantomData<R>,
}

impl<R: SharedResource> AddSystemToApp for ResourceUpdater<R> {
    fn add_system_to_app(&self, app: &mut App) {
        app.add_systems(Update, systems::update_shared_resource::<R>);
    }
}

/// Add this plugin to your Bevy app to enable WASM-based modding
///
/// Give [`WasmPlugin::new`] a list of wasm files to load at startup.
/// Further mods can be added at any time with [`WasmMod::new()`].
pub struct WasmPlugin<In, Out>
where
    In: Message,
    Out: Message,
{
    protocol_version: Version,
    shared_resources: Vec<Box<dyn AddSystemToApp>>,
    _in: std::marker::PhantomData<In>,
    _out: std::marker::PhantomData<Out>,
}

impl<In: Message, Out: Message> WasmPlugin<In, Out> {
    /// Create a WasmPlugin with a list of wasm files to load at startup
    pub fn new(protocol_version: Version) -> Self {
        info!(
            "Starting {}{}{}{} {}{}{}{} with protocol version {}.{}.{}",
            "B".bold().red(),
            "E".bold().yellow(),
            "V".bold().green(),
            "Y".bold().cyan(),
            "W".bold().blue(),
            "A".bold().magenta(),
            "S".bold().red(),
            "M".bold().yellow(),
            protocol_version.major,
            protocol_version.minor,
            protocol_version.patch,
        );
        WasmPlugin {
            protocol_version,
            shared_resources: Vec::new(),
            _in: std::marker::PhantomData,
            _out: std::marker::PhantomData,
        }
    }

    /// Register a resource to be shared with mods. THIS SHOULD COME FROM YOUR PROTOCOL CRATE
    pub fn share_resource<T: SharedResource>(mut self) -> Self {
        self.shared_resources.push(Box::new(ResourceUpdater::<T> {
            _r: std::marker::PhantomData,
        }));
        self
    }
}

impl<In: Message, Out: Message> Plugin for WasmPlugin<In, Out> {
    fn build(&self, app: &mut App) {
        let wasm_resource = WasmRuntime::new(self.protocol_version);

        app.insert_resource(wasm_resource)
            .init_asset::<WasmAsset>()
            .init_asset_loader::<WasmAssetLoader>()
            .add_event::<In>()
            .add_event::<Out>()
            .add_systems(Update, load_instances)
            .add_systems(Update, systems::tick_mods::<In, Out>);

        for system in self.shared_resources.iter() {
            system.add_system_to_app(app);
        }
    }
}
