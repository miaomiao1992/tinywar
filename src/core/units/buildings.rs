use crate::core::settings::PlayerColor;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum BuildingName {
    #[default]
    Castle,
}

impl BuildingName {
    pub fn units(&self) -> Vec<Vec2> {
        match self {
            BuildingName::Castle => {
                vec![Vec2::new(-70., 35.), Vec2::new(0., 20.), Vec2::new(70., 35.)]
            },
        }
    }

    pub fn health(&self) -> f32 {
        match self {
            BuildingName::Castle => 250.,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Building {
    pub name: BuildingName,
    pub color: PlayerColor,
    pub is_base: bool,
    pub health: f32,
}

impl Building {
    pub fn new(name: BuildingName, color: PlayerColor, is_base: bool) -> Self {
        Self {
            name,
            color,
            is_base,
            health: name.health(),
        }
    }
}
