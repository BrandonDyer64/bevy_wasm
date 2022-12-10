use serde::{Deserialize, Serialize};

/// A message to be sent Mod -> Game.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ModMessage {
    Hello,
}

/// A message to be sent Game -> Mod.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum GameMessage {
    HiThere,
}
