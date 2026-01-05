use crate::core::settings::PlayerColor;
use crate::core::units::buildings::{Building, BuildingName};
use crate::core::units::soldiers::{Soldier, SoldierName};
use bevy::prelude::*;
use bevy_renet::renet::ClientId;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub id: ClientId,
    pub color: PlayerColor,
    pub buildings: Vec<Building>,
    pub soldiers: Vec<Soldier>,
    pub queue: VecDeque<SoldierName>,
    pub queue_timer: Duration,
}

impl Player {
    pub fn new(id: ClientId, color: PlayerColor, pos: Vec2) -> Self {
        Self {
            id,
            color,
            buildings: vec![Building::new(BuildingName::Castle, pos)],
            soldiers: vec![],
            queue: VecDeque::new(),
            queue_timer: Duration::default(),
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
}
