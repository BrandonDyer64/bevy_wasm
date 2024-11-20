//! The plugin for your mod

use std::ffi::c_void;

use bevy::app::{App, AppExit, Plugin, Update};
use bevy::ecs::{
    event::Event,
    prelude::{EventReader, EventWriter},
    system::ResMut,
};
use bevy_wasm_shared::prelude::*;
use serde::{de::DeserializeOwned, Serialize};
use bevy_wasm_sys_core::events::{get_next_event, send_event};
use bevy_wasm_sys_core::{error, info};

use crate::{
    ecs::extern_res::ExternResources,
    ffi::store_app,
    time::Time,
};

/// An object that can be used as a message
pub trait Message: Send + Sync + Serialize + DeserializeOwned + Event + 'static {}

impl<T> Message for T where T: Send + Sync + Serialize + DeserializeOwned + Event + 'static {}

/// Use this plugin in your app to enable communication with the host
///
/// Necessary for modding support
///
/// - In: Message type going from host to mod
/// - Out: Message type going from mod to host
pub struct FFIPlugin<In: Message, Out: Message> {
    protocol_version: Version,
    protocol_version_checker: Box<dyn Fn(Version, Version) -> bool + Send + Sync + 'static>,
    _in: std::marker::PhantomData<In>,
    _out: std::marker::PhantomData<Out>,
}

impl<In: Message, Out: Message> FFIPlugin<In, Out> {
    /// Create a new FFIPlugin instance to insert into a Bevy `App`
    pub fn new(protocol_version: Version) -> Self {
        info!(
            "Starting mod with protocol version {}.{}.{}",
            protocol_version.major, protocol_version.minor, protocol_version.patch
        );
        Self {
            protocol_version,
            protocol_version_checker: Box::new(|host_version, mod_version| {
                // Check that the names match
                host_version.name_hash == mod_version.name_hash
                // Check that the major versions match
                && host_version.major == mod_version.major
            }),
            _in: std::marker::PhantomData,
            _out: std::marker::PhantomData,
        }
    }

    /// Set a custom protocol version checker to ensure your mod is compatible with the game
    ///
    /// The default version checker only ensures the major versions match.
    ///
    /// i.e. `1.0.0` is compatible with `1.1.0` but not `2.0.0`
    pub fn with_version_checker<F>(self, checker: F) -> Self
    where
        F: Fn(Version, Version) -> bool + Send + Sync + 'static,
    {
        Self {
            protocol_version_checker: Box::new(checker),
            ..self
        }
    }
}

impl<In: Message, Out: Message> Plugin for FFIPlugin<In, Out> {
    fn build(&self, app: &mut App) {
        let host_version = unsafe { crate::ffi::get_protocol_version() };
        let host_version = Version::from_u64(host_version);
        if !(*self.protocol_version_checker)(self.protocol_version, host_version) {
            error!(
                "Protocol version incompatible! Host: {}.{}.{}, Mod: {}.{}.{}",
                host_version.major,
                host_version.minor,
                host_version.patch,
                self.protocol_version.major,
                self.protocol_version.minor,
                self.protocol_version.patch
            );
            return;
        }
        app.set_runner(app_runner)
            .add_event::<In>()
            .add_event::<Out>()
            .insert_resource(Time::new())
            .insert_resource(ExternResources::new())
            .add_systems(Update, update_time)
            .add_systems(Update, fetch_resources)
            .add_systems(Update, event_listener::<In>)
            .add_systems(Update, event_sender::<Out>);
        // .add_system_to_stage(CoreStage::First, update_time.at_start())
        // .add_system_to_stage(CoreStage::PreUpdate, fetch_resources)
        // .add_system_to_stage(CoreStage::PreUpdate, event_listener::<In>)
        // .add_system_to_stage(CoreStage::PostUpdate, event_sender::<Out>);
    }
}

fn fetch_resources(mut resources: ResMut<ExternResources>) {
    resources.fetch_all();
}

fn event_listener<M: Message>(mut events: EventWriter<M>) {
    while let Some(event) = get_next_event() {
        events.send(event);
    }
}

fn event_sender<M: Message>(mut events: EventReader<M>) {
    for event in events.read() {
        send_event(event);
    }
}

fn app_runner(app: App) -> AppExit {
    let app = Box::new(app);
    let app_ptr = Box::into_raw(app);
    let app_ptr = app_ptr as *const c_void;
    unsafe { store_app(app_ptr) };
    AppExit::Success
}

fn update_time(mut time: ResMut<Time>) {
    time.update();
}
