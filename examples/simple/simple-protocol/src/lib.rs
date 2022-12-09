use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum HostMessage {
    SaySomething(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ModMessage {
    SaySomething(String),
}
