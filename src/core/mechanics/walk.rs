use crate::core::constants::{CAPPED_DELTA_SECS_SPEED, RADIUS};
use crate::core::map::map::Map;
use crate::core::player::Players;
use crate::core::settings::Settings;
use crate::core::units::units::{Action, Unit, UnitName};
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::TilePos;
use std::collections::HashMap;

/// Return the next tile to walk to, which is the one following the closest tile
fn next_tile_on_path<'a>(pos: &Vec3, path: &'a Vec<TilePos>) -> &'a TilePos {
    path.iter()
        .enumerate()
        .min_by_key(|(_, tile)| {
            let tile_pos = Map::tile_to_world(tile);
            let dx = pos.x - tile_pos.x;
            let dy = pos.y - tile_pos.y;
            (dx * dx + dy * dy) as i32
        })
        .map(|(idx, _)| path.get(idx + 1).unwrap_or(path.last().unwrap()))
        .unwrap()
}

pub fn run(
    unit_e: Entity,
    unit: &mut Unit,
    unit_t: &mut Transform,
    unit_s: &mut Sprite,
    positions: &HashMap<TilePos, Vec<(Entity, Vec3, Unit)>>,
    settings: &Settings,
    map: &Map,
    players: &Players,
    time: &Time,
) {
    let tile = Map::world_to_tile(&unit_t.translation);
    let mut path = map.path(&unit.path);

    // Reverse paths for the enemy
    if players.me.color != unit.color {
        path.reverse();
    }

    if tile == *path.last().unwrap() {
        unit.action = Action::Idle;
        return;
    }

    let target_tile = next_tile_on_path(&unit_t.translation, &path);
    let target_pos = Map::tile_to_world(&target_tile).extend(unit_t.translation.z);

    // Check units in this tile + adjacent tiles
    for tile in std::iter::once(tile).chain(Map::get_neighbors(&tile)) {
        if let Some(units) = positions.get(&tile) {
            for (other_e, other_pos, other_unit) in units {
                if unit_e == *other_e {
                    continue; // Skip self
                }

                let delta = unit_t.translation - other_pos;
                let dist = delta.length();

                let range = unit.name.range() * RADIUS;
                if unit.color != other_unit.color {
                    // Resolve if encountered enemy
                    if dist < range {
                        unit.action = if unit.name != UnitName::Monk {
                            Action::Attack(*other_e)
                        } else {
                            Action::Idle // Monk's can't attack
                        };
                        return;
                    }
                } else {
                    // Resolve if encountered ally
                    if unit.name == UnitName::Monk
                        && dist < range
                        && other_unit.health < other_unit.name.health()
                    {
                        unit.action = Action::Heal(*other_e);
                        return;
                    }
                }
            }
        }
    }

    let mut next_pos = unit_t.translation
        + (target_pos - unit_t.translation).normalize()
            * unit.name.speed()
            * settings.speed
            * time.delta_secs().min(CAPPED_DELTA_SECS_SPEED);
    let next_tile = Map::world_to_tile(&next_pos);

    if tile == next_tile || Map::is_walkable(&next_tile) {
        // Check if the tile below is walkable. If not, restrict movement to the top part
        if !Map::is_walkable(&TilePos::new(next_tile.x, next_tile.y + 1)) {
            let bottom_limit = Map::tile_to_world(&next_tile).y - Map::TILE_SIZE as f32 * 0.25;
            if next_pos.y < bottom_limit {
                next_pos.y = bottom_limit;
            }
        }

        // Change the direction the unit is facing
        // unit_s.flip_x = next_pos.x < unit_t.translation.x;

        unit_t.translation = next_pos;
        unit.action = Action::Run;
    } else {
        unit.action = Action::Idle;
    }
}

pub fn move_units(
    mut unit_q: Query<(Entity, &mut Transform, &mut Sprite, &mut Unit)>,
    settings: Res<Settings>,
    map: Res<Map>,
    players: Res<Players>,
    time: Res<Time>,
) {
    // Build spatial hashmap: tile -> positions + unit
    let positions: HashMap<TilePos, Vec<(Entity, Vec3, Unit)>> =
        unit_q.iter().fold(HashMap::new(), |mut acc, (e, t, _, u)| {
            let tile = Map::world_to_tile(&t.translation);
            acc.entry(tile).or_default().push((e, t.translation, u.clone()));
            acc
        });

    for (unit_e, mut unit_t, mut unit_s, mut unit) in
        unit_q.iter_mut().filter(|(_, _, _, u)| matches!(u.action, Action::Idle | Action::Run))
    {
        run(
            unit_e,
            &mut unit,
            &mut unit_t,
            &mut unit_s,
            &positions,
            &settings,
            &map,
            &players,
            &time,
        );
    }
}
