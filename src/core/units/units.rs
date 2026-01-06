use crate::core::settings::PlayerColor;
use bevy::prelude::{Component, KeyCode};
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(EnumIter, Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum UnitName {
    #[default]
    Warrior,
    Lancer,
}

impl UnitName {
    pub fn key(&self) -> KeyCode {
        match self {
            UnitName::Warrior => KeyCode::KeyZ,
            UnitName::Lancer => KeyCode::KeyX,
        }
    }

    /// Spawning time in milliseconds
    pub fn spawn_duration(&self) -> u64 {
        match self {
            UnitName::Warrior => 2000,
            UnitName::Lancer => 1000,
        }
    }

    pub fn size(&self) -> f32 {
        match self {
            UnitName::Warrior => 192.,
            UnitName::Lancer => 320.,
        }
    }

    pub fn scale(&self) -> f32 {
        match self {
            UnitName::Warrior => 0.5,
            UnitName::Lancer => 0.45,
        }
    }

    pub fn frames(&self, action: Action) -> u32 {
        match self {
            UnitName::Warrior => match action {
                Action::Idle => 8,
            },
            UnitName::Lancer => match action {
                Action::Idle => 12,
            },
        }
    }
}

#[derive(EnumIter, Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum Action {
    #[default]
    Idle,
}

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Unit {
    pub name: UnitName,
    pub color: PlayerColor,
    pub action: Action,
}

impl Unit {
    pub fn new(name: UnitName, color: PlayerColor, action: Action) -> Self {
        Unit {
            name,
            color,
            action,
        }
    }
}
