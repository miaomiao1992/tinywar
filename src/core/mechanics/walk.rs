use crate::core::constants::{CAPPED_DELTA_SECS_SPEED, UNIT_DEFAULT_SIZE, UNIT_SCALE};
use crate::core::map::map::Map;
use crate::core::player::Players;
use crate::core::settings::Settings;
use crate::core::units::units::{Action, Unit, UnitName};
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::TilePos;
use std::collections::HashMap;

fn closest_tile_on_path<'a>(current_tile: &TilePos, path: &'a Vec<TilePos>) -> &'a TilePos {
    let mut nearest_idx = 0;
    let mut min_dist = f32::MAX;

    for (i, tile) in path.iter().enumerate() {
        let dx = current_tile.x as i32 - tile.x as i32;
        let dy = current_tile.y as i32 - tile.y as i32;
        let dist_sq = (dx * dx + dy * dy) as f32;

        if dist_sq < min_dist {
            min_dist = dist_sq;
            nearest_idx = i;
        }
    }

    // Return the next tile after the nearest one, or the last tile if at the end
    path.get(nearest_idx + 1).unwrap_or(path.last().unwrap())
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

    let target_tile = closest_tile_on_path(&tile, &path);

    // Calculate the vector to the next location
    let target_pos = Map::tile_to_world(&target_tile).extend(unit_t.translation.z);

    let direction = target_pos - unit_t.translation;

    let radius = UNIT_DEFAULT_SIZE * 0.5 * UNIT_SCALE;

    // Check units in this tile + adjacent tiles
    for tile in std::iter::once(tile).chain(Map::get_neighbors(&tile)) {
        if let Some(units) = positions.get(&tile) {
            for (other_e, other_pos, other_unit) in units {
                if unit_e == *other_e {
                    continue; // Skip self
                }

                let delta = unit_t.translation - other_pos;
                let dist = delta.length();

                let range = unit.name.range() * radius;
                if unit.color != other_unit.color {
                    // Resolve if encountered enemy
                    if dist < range {
                        unit.action = if unit.name != UnitName::Monk {
                            Action::Attack
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
                        unit.action = Action::Heal;
                        return;
                    }
                }
            }
        }
    }

    let mut next_pos = unit_t.translation
        + direction.normalize()
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
        unit_s.flip_x = next_pos.x < unit_t.translation.x;

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
