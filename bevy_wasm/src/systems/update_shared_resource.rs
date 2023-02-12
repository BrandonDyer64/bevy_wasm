use std::{ops::Deref, sync::Arc};

use bevy::prelude::*;
use serde::{de::DeserializeOwned, Serialize};

use crate::{components::WasmMod, Message};

pub fn update_shared_resource<
    T: Resource + DeserializeOwned + Serialize,
    In: Message,
    Out: Message,
>(
    res: Res<T>,
    mut wasm_mods: Query<&mut WasmMod>,
) {
    if res.is_changed() {
        let v: &T = res.deref();
        let resource_bytes: Arc<[u8]> = bincode::serialize(v).unwrap().into();
        let resource_name = std::any::type_name::<T>().to_string();
        for mut wasm_mod in wasm_mods.iter_mut() {
            wasm_mod.update_resource_value(&resource_name, resource_bytes.clone());
        }
    }
}
