//! The plugin for your mod

use std::ffi::c_void;

use bevy_app::{App, CoreStage, Plugin};
use bevy_ecs::prelude::{EventReader, EventWriter};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    events::{get_next_event, send_event},
    ffi::store_app,
};

/// An object that can be used as a message
pub trait Message: Send + Sync + Serialize + DeserializeOwned + 'static {}

impl<T> Message for T where T: Send + Sync + Serialize + DeserializeOwned + 'static {}

/// Use this plugin in your app to enable communication with the host
///
/// Necessary for modding support
///
/// - In: Message type going from host to mod
/// - Out: Message type going from mod to host
pub struct FFIPlugin<In: Message, Out: Message>(
    std::marker::PhantomData<In>,
    std::marker::PhantomData<Out>,
);

impl<In: Message, Out: Message> Default for FFIPlugin<In, Out> {
    fn default() -> Self {
        Self(std::marker::PhantomData, std::marker::PhantomData)
    }
}

impl<In: Message, Out: Message> Plugin for FFIPlugin<In, Out> {
    fn build(&self, app: &mut App) {
        app.set_runner(app_runner)
            .add_event::<In>()
            .add_event::<Out>()
            .add_system_to_stage(CoreStage::PreUpdate, event_listener::<In>)
            .add_system_to_stage(CoreStage::PostUpdate, event_sender::<Out>);
    }
}

fn event_listener<M: Message>(mut events: EventWriter<M>) {
    while let Some(event) = get_next_event() {
        events.send(event);
    }
}

fn event_sender<M: Message>(mut events: EventReader<M>) {
    for event in events.iter() {
        send_event(event);
    }
}

fn app_runner(mut app: App) {
    app.update();
    let app = Box::new(app);
    let app_ptr = Box::into_raw(app);
    let app_ptr = app_ptr as *const c_void;
    unsafe { store_app(app_ptr) };
}
