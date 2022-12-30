//! Access host `Resource`s from inside of a WASM system

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
    ops::Deref,
};

use bevy_ecs::{prelude::*, system::SystemParam};
use serde::{de::DeserializeOwned, Serialize};

use crate::error;

/// Get the value of a resource from the host
pub fn get_resource<T: Resource + DeserializeOwned>() -> Option<T> {
    let name = std::any::type_name::<T>();
    let mut buffer = [0; 1024];

    let len = unsafe {
        // put serialized resource into buffer
        crate::ffi::get_resource(name.as_ptr(), name.len(), buffer.as_mut_ptr(), buffer.len())
    };

    if len == 0 {
        return None;
    }

    if len > buffer.len() {
        error!("Serialized resource is larger than buffer");
        return None;
    }

    match bincode::deserialize(&buffer[..len]) {
        Ok(resource) => Some(resource),
        Err(err) => {
            error!("Failed to deserialize resource from host: {}", err);
            None
        }
    }
}

trait ResourceFetch: Send + Sync {
    fn fetch(&mut self) -> Option<Box<dyn Any + Send + Sync>>;
}

struct ExternResourceFetcher<T: Resource + DeserializeOwned + Send + Sync>(PhantomData<T>);

impl<T: Resource + DeserializeOwned + Send + Sync> ResourceFetch for ExternResourceFetcher<T> {
    fn fetch(&mut self) -> Option<Box<dyn Any + Send + Sync>> {
        Some(Box::new(get_resource::<T>()?))
    }
}

struct ExternResourceValue {
    value: Box<dyn Any + Send + Sync>,
    fetcher: Box<dyn ResourceFetch>,
}

impl ExternResourceValue {
    pub fn init<T: Resource + Serialize + DeserializeOwned>() -> Self {
        Self {
            value: Box::new(match get_resource::<T>() {
                Some(v) => v,
                None => {
                    error!(
                        "Resource was not shared with mod: {}",
                        std::any::type_name::<T>()
                    );
                    panic!();
                }
            }),
            fetcher: Box::new(ExternResourceFetcher::<T>(PhantomData)),
        }
    }

    fn fetch(&mut self) {
        if let Some(new_value) = self.fetcher.fetch() {
            self.value = Box::new(new_value);
        }
    }

    fn try_get<T: Resource>(&self) -> Option<&T> {
        self.value.downcast_ref()
    }
}

#[doc(hidden)]
#[derive(Resource)]
pub struct ExternResources {
    resources: HashMap<TypeId, ExternResourceValue>,
}

impl ExternResources {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub fn insert<T: Resource + Serialize + DeserializeOwned>(&mut self) {
        self.resources
            .insert(TypeId::of::<T>(), ExternResourceValue::init::<T>());
    }

    pub fn fetch_all(&mut self) {
        for resource_value in self.resources.values_mut() {
            resource_value.fetch();
        }
    }

    pub fn try_get<T: Resource + Serialize + DeserializeOwned>(&self) -> Option<&T> {
        self.resources.get(&TypeId::of::<T>())?.try_get()
    }
}

/// Use a resource from the host game
#[derive(SystemParam)]
pub struct ExternRes<'w, 's, T: Resource + Serialize + DeserializeOwned> {
    res: Res<'w, ExternResources>,
    #[system_param(ignore)]
    t: PhantomData<T>,
    #[system_param(ignore)]
    marker: PhantomData<&'s ()>,
}

impl<'w, 's, T: Resource + Serialize + DeserializeOwned> ExternRes<'w, 's, T> {
    /// Get the resource
    pub fn try_get(&self) -> Option<&T> {
        self.res.resources.get(&TypeId::of::<T>())?.try_get()
    }
}

impl<'w, 's, T: Resource + Serialize + DeserializeOwned> Deref for ExternRes<'w, 's, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self.try_get() {
            Some(v) => v,
            None => {
                error!(
                    "Resource was not shared with mod: {}",
                    std::any::type_name::<T>()
                );
                panic!();
            }
        }
    }
}

/// Convenience re-exports
pub mod prelude {
    pub use super::ExternRes;
}
