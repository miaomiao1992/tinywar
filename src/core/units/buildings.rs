use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Debug, Serialize, Deserialize)]
pub enum BuildingName {
    Barracks,
    Castle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Building {
    pub name: BuildingName,
    pub position: Vec2,
}

impl Building {
    pub fn new(name: BuildingName, position: Vec2) -> Self {
        Self {
            name,
            position,
        }
    }
}
