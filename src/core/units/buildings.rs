use crate::core::settings::PlayerColor;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum BuildingName {
    #[default]
    Castle,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Building {
    pub name: BuildingName,
    pub color: PlayerColor,
    pub is_base: bool,
}

impl Building {
    pub fn new(name: BuildingName, color: PlayerColor, is_base: bool) -> Self {
        Self {
            name,
            color,
            is_base,
        }
    }
}
