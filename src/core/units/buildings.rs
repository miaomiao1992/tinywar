use crate::core::settings::{PlayerColor, Settings};
use bevy::prelude::*;
use std::ops::{Deref, DerefMut};
use strum_macros::EnumIter;
use crate::core::constants::GRID_SIZE;

#[derive(EnumIter, Debug)]
pub enum BuildingName {
    Barracks,
    Castle,
}

#[derive(Debug)]
pub struct Building {
    pub name: BuildingName,
    pub color: PlayerColor,
    pub position: Vec2,
}

impl Building {
    pub fn new(name: BuildingName, color: PlayerColor, position: Vec2) -> Self {
        Self {
            name,
            color,
            position,
        }
    }
}

#[derive(Resource)]
pub struct Buildings(pub Vec<Building>);

impl Buildings {
    pub fn new(settings: &Settings) -> Self {
        let coord = |pos: (f32, f32)| Vec2::new(GRID_SIZE * pos.0, GRID_SIZE * pos.1);

        // Get the coordinates to place the starting buildings
        let positions = settings.map_size.starting_positions();

        Self(vec![
            Building::new(BuildingName::Castle, settings.color, coord(positions[0])),
            Building::new(BuildingName::Castle, settings.enemy_color, coord(positions[1])),
        ])
    }
}

impl Deref for Buildings {
    type Target = Vec<Building>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Buildings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
