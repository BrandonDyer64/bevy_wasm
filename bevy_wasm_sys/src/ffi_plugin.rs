use std::ffi::c_void;

use bevy_app::{App, CoreStage, Plugin};
use bevy_ecs::prelude::{EventReader, EventWriter};
use serde::{de::DeserializeOwned, Serialize};

use crate::{error, events::send_event, ffi::store_app, info};

pub struct FFIPlugin<M: Send + Sync + Serialize + DeserializeOwned + 'static>(
    std::marker::PhantomData<M>,
);

impl<M: Send + Sync + Serialize + DeserializeOwned + 'static> Default for FFIPlugin<M> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<M: Send + Sync + Serialize + DeserializeOwned + 'static> Plugin for FFIPlugin<M> {
    fn build(&self, app: &mut App) {
        app.set_runner(app_runner)
            .add_event::<M>()
            .add_system_to_stage(CoreStage::PreUpdate, event_listener::<M>)
            .add_system_to_stage(CoreStage::PostUpdate, event_sender::<M>);
    }
}

fn event_listener<M: Send + Sync + DeserializeOwned + 'static>(mut events: EventWriter<M>) {
    // let event_values_raw = vec![]; // TODO
    // for event_value_raw in event_values_raw {
    //     let event_value = match bincode::deserialize(event_value_raw) {
    //         Ok(event_value) => event_value,
    //         Err(err) => {
    //             error!("Failed to deserialize event: {}", err);
    //             continue;
    //         }
    //     };
    //     events.send(event_value);
    // }
}

fn event_sender<M: Send + Sync + Serialize + 'static>(mut events: EventReader<M>) {
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
