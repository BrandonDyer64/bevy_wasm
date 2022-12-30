//! Add this plugin to your Bevy app to enable WASM-based modding

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_log::prelude::*;
use bevy_wasm_shared::prelude::*;
use colored::*;
use serde::{de::DeserializeOwned, Serialize};

use crate::{resource::WasmResource, systems::*, Message};

trait AddSystemToApp: Send + Sync + 'static {
    fn add_system_to_app(&self, app: &mut App);
}

struct ResourceUpdater<R: Resource + Serialize + DeserializeOwned, In: Message, Out: Message> {
    _r: std::marker::PhantomData<R>,
    _in: std::marker::PhantomData<In>,
    _out: std::marker::PhantomData<Out>,
}

impl<R: Resource + Serialize + DeserializeOwned, In: Message, Out: Message> AddSystemToApp
    for ResourceUpdater<R, In, Out>
{
    fn add_system_to_app(&self, app: &mut App) {
        app.add_system(update_shared_resource::<R, In, Out>);
    }
}

/// Add this plugin to your Bevy app to enable WASM-based modding
///
/// Give [`WasmPlugin::new`] a list of wasm files to load at startup.
/// Further mods can be added at any time with [`WasmResource::insert_wasm`].
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
            protocol_version: protocol_version.into(),
            shared_resources: Vec::new(),
            _in: std::marker::PhantomData,
            _out: std::marker::PhantomData,
        }
    }

    /// Register a resource to be shared with mods. THIS SHOULD COME FROM YOUR PROTOCOL CRATE
    pub fn share_resource<T: Resource + Serialize + DeserializeOwned>(mut self) -> Self {
        self.shared_resources
            .push(Box::new(ResourceUpdater::<T, In, Out> {
                _r: std::marker::PhantomData,
                _in: std::marker::PhantomData,
                _out: std::marker::PhantomData,
            }));
        self
    }
}

impl<In: Message, Out: Message> Plugin for WasmPlugin<In, Out> {
    fn build(&self, app: &mut App) {
        let wasm_resource = WasmResource::<In, Out>::new(self.protocol_version.clone().into());

        app.insert_resource(wasm_resource)
            .add_event::<In>()
            .add_event::<Out>()
            .add_system(update_mods::<In, Out>)
            .add_system_to_stage(CoreStage::PostUpdate, event_listener::<In, Out>);

        for system in self.shared_resources.iter() {
            system.add_system_to_app(app);
        }
    }
}
