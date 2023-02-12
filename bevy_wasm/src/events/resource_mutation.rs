use bevy::prelude::*;

/// A resource mutation request
///
/// For when a mod wants a new value for a resource
pub struct ResourceMutation<T: Resource> {
    /// The new value for the resource
    pub proposed_value: T,

    /// The mod that requested the change
    pub requester: Entity,
}
