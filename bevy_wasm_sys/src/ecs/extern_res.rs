//! Access host `Resource`s from inside of a WASM system

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    marker::PhantomData,
    ops::Deref,
};

use bevy_ecs::{prelude::*, system::SystemParam};
use serde::{de::DeserializeOwned, Serialize};

use crate::error;

/// Get the value of a resource from the host
pub fn get_resource<T: Resource + Serialize + DeserializeOwned>() -> Option<T> {
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

    let resource_bytes = &buffer[..len];

    match bincode::deserialize(resource_bytes) {
        Ok(resource) => Some(resource),
        Err(err) => {
            error!("Failed to deserialize resource from host: {}", err);
            None
        }
    }
}

trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

trait AnyResource: AsAny + Any + Resource + Send + Sync + 'static {}

impl<T: Any + Resource + Send + Sync + 'static> AnyResource for T {}

impl dyn AnyResource {
    fn downcast_ref<T: AnyResource>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }
}

trait ResourceFetch: Send + Sync {
    fn fetch(&mut self) -> Option<Box<dyn AnyResource>>;
}

struct ExternResourceFetchImpl<T: Resource + DeserializeOwned + Send + Sync>(PhantomData<T>);

impl<T: Resource + Serialize + DeserializeOwned + Send + Sync> ResourceFetch
    for ExternResourceFetchImpl<T>
{
    fn fetch(&mut self) -> Option<Box<dyn AnyResource>> {
        Some(Box::new(get_resource::<T>()?))
    }
}

struct ExternResourceValue {
    value: Option<Box<dyn AnyResource>>,
    fetcher: Box<dyn ResourceFetch>,
}

impl ExternResourceValue {
    pub fn init<T: Resource + Serialize + DeserializeOwned>() -> Self {
        Self {
            value: match get_resource::<T>() {
                Some(v) => Some(Box::new(v)),
                None => None,
            },
            fetcher: Box::new(ExternResourceFetchImpl::<T>(PhantomData)),
        }
    }

    pub fn fetch(&mut self) {
        if let Some(new_value) = self.fetcher.fetch() {
            self.value = Some(new_value);
        }
    }

    pub fn downcast_ref<T: Resource + Serialize + DeserializeOwned>(&self) -> Option<&T> {
        self.value
            .as_ref()
            .and_then(|boxed| (&**boxed as &(dyn AnyResource + 'static)).downcast_ref::<T>())
    }
}

#[doc(hidden)]
#[derive(Resource)]
pub struct ExternResources {
    resources: HashMap<TypeId, ExternResourceValue>,
}

impl Debug for ExternResources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_map();
        for (type_id, resource_value) in &self.resources {
            debug.entry(&type_id, &resource_value.value.is_some());
        }
        debug.finish()
    }
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

    pub fn get<T: Resource + Serialize + DeserializeOwned>(&self) -> Option<&T> {
        self.resources.get(&TypeId::of::<T>())?.downcast_ref()
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

impl<'w, 's, T: Debug + Resource + Serialize + DeserializeOwned> Debug for ExternRes<'w, 's, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

impl<'w, 's, T: Resource + Serialize + DeserializeOwned> ExternRes<'w, 's, T> {
    /// Get the resource
    pub fn get(&self) -> Option<&T> {
        self.res.get::<T>()
    }
}

impl<'w, 's, T: Resource + Serialize + DeserializeOwned> Deref for ExternRes<'w, 's, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self.get() {
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
