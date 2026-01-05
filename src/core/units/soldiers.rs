use bevy::prelude::KeyCode;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum SoldierName {
    #[default]
    Warrior,
}

impl SoldierName {
    pub fn key(&self) -> KeyCode {
        match self {
            SoldierName::Warrior => KeyCode::KeyZ,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Soldier {
    pub name: SoldierName,
}
