use crate::core::settings::PlayerColor;
use crate::core::units::units::UnitName;
use bevy::prelude::*;
use bevy_renet::renet::ClientId;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::Duration;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum PlayerDirection {
    #[default]
    Any,
    Top,
    TopMid,
    Mid,
    MidBot,
    Bot,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueuedUnit {
    pub unit: UnitName,
    pub timer: Timer,
}

impl QueuedUnit {
    pub fn new(unit: UnitName, millis: u64) -> QueuedUnit {
        Self {
            unit,
            timer: Timer::new(Duration::from_millis(millis), TimerMode::Once),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub id: ClientId,
    pub color: PlayerColor,
    pub direction: PlayerDirection,
    pub queue: VecDeque<QueuedUnit>,
    pub queue_default: UnitName,
}

impl Player {
    pub fn new(id: ClientId, color: PlayerColor) -> Self {
        Self {
            id,
            color,
            direction: PlayerDirection::default(),
            queue: VecDeque::new(),
            queue_default: UnitName::default(),
        }
    }

    pub fn is_human(&self) -> bool {
        self.id == 0 || (self.id > 10 && self.id < ClientId::MAX)
    }
}

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct Players {
    pub me: Player,
    pub enemy: Player,
}

impl Players {
    pub fn get(&self, id: ClientId) -> &Player {
        if self.me.id == id {
            &self.me
        } else {
            &self.enemy
        }
    }

    pub fn get_mut(&mut self, id: ClientId) -> &mut Player {
        if self.me.id == id {
            &mut self.me
        } else {
            &mut self.enemy
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Player> {
        [&self.me, &self.enemy].into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Player> {
        [&mut self.me, &mut self.enemy].into_iter()
    }
}
