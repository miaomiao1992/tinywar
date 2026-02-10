use crate::core::boosts::Boost;
use crate::core::constants::*;
use crate::core::map::map::Map;
use crate::core::mechanics::combat::{ApplyDamageMsg, Arrow, ProjectileMode};
use crate::core::mechanics::spawn::DespawnMsg;
use crate::core::player::{Players, Side, Strategy};
use crate::core::settings::{PlayerColor, Settings};
use crate::core::units::buildings::Building;
use crate::core::units::units::{Action, Unit, UnitName};
use crate::utils::scale_duration;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::TilePos;
use std::collections::{HashMap, HashSet};

/// Get all tiles at <= `distance` from `pos`
fn get_tiles_at_distance(pos: &TilePos, d: u32) -> HashSet<TilePos> {
    (pos.x.saturating_sub(d)..=pos.x + d)
        .flat_map(|x| (pos.y.saturating_sub(d)..=pos.y + d).map(move |y| TilePos::new(x, y)))
        .collect()
}

/// Return the next tile to walk to, which is the one after the closest lane tile
fn get_target_tile(tile: TilePos, path: &[TilePos]) -> TilePos {
    let pos = Map::tile_to_world(tile);
    let closest = path
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            let da = Map::tile_to_world(**a).distance(pos);
            let db = Map::tile_to_world(**b).distance(pos);
            da.partial_cmp(&db).unwrap()
        })
        .and_then(|(i, _)| path.get(i + 1).or_else(|| path.last()))
        .unwrap();

    Map::find_path(tile, *closest)[1]
}

fn move_unit(
    unit_e: Entity,
    unit: &mut Unit,
    unit_t: &mut Transform,
    unit_s: &mut Sprite,
    unit_pos: &HashMap<TilePos, Vec<(Entity, Vec3, Unit)>>,
    building_pos: &HashMap<TilePos, Vec<(Entity, Vec3, Building)>>,
    settings: &Settings,
    map: &Map,
    players: &Players,
    time: &Time,
) {
    let tile = Map::world_to_tile(&unit_t.translation);
    let mut lane = map.lane(&unit.lane);

    let player = players.get_by_color(unit.color);
    let enemy = players.get_by_side(player.side.opposite());

    // Reverse paths for the enemy
    if player.side == Side::Right {
        lane.reverse();
    }

    if tile == *lane.last().unwrap() {
        unit.action = Action::Idle;
        return;
    }

    let target_tile = get_target_tile(tile, &lane);
    let target_pos = Map::tile_to_world(target_tile).extend(unit_t.translation.z);
    let target_delta = (target_pos - unit_t.translation).normalize();

    let mut separation = Vec3::ZERO;

    // Only check units in the surrounding
    for tile in get_tiles_at_distance(&tile, 5) {
        if let Some(units) = unit_pos.get(&tile) {
            'enemy: for (other_e, other_pos, other) in units {
                let delta = unit_t.translation - other_pos;
                let dist = delta.length();

                // Distance between the units to perform melee attacks
                let separation_radius = (unit.name.world_size() + other.name.world_size()) * 0.15;

                let range = if unit.name.is_melee() {
                    separation_radius
                } else {
                    unit.range(player) * RADIUS
                };

                // Skip if self or too far to interact
                if unit_e == *other_e || dist > range {
                    continue;
                }

                // Possible interactions are:
                // - Priest with unhealthy ally -> heal
                // - Combat unit with enemy -> attack
                // - Else: resolve separation force
                unit.action = match (unit.name, unit.color == other.color) {
                    (UnitName::Priest, true) if other.health < other.name.health() => {
                        Action::Heal(*other_e)
                    },
                    (u, false)
                        if u.can_attack()
                            && (!u.is_melee() || dist <= separation_radius)
                            && player.strategy != Strategy::March =>
                    {
                        Action::Attack(*other_e)
                    },
                    _ => {
                        if dist <= separation_radius
                            && other.on_building.is_none()
                            && !player.has_boost(Boost::NoCollision)
                        {
                            // Handle units on top of each other
                            let delta_norm = if dist < 0.01 {
                                // Use a deterministic direction based on entity to ensure consistent separation
                                let angle = unit_e.to_bits() as f32 * 10.;
                                Vec3::new(angle.cos(), angle.sin(), 0.)
                            } else {
                                delta.normalize()
                            };

                            let strength = (separation_radius - dist).powi(3) / (separation_radius);

                            // Calculate a "sideways" vector (perpendicular to movement)
                            let perpendicular = Vec3::new(-target_delta.y, target_delta.x, 0.);

                            // Determine which side of the path the other unit is on
                            let sign = if delta_norm.dot(perpendicular) >= 0. {
                                1.
                            } else {
                                -1.
                            };

                            // Apply force: mostly perpendicular to bypass, slightly away to avoid collision
                            separation += perpendicular * sign + delta_norm * strength;
                        }

                        continue 'enemy;
                    },
                };

                return; // The unit's action is non-run -> skip movement
            }
        }

        if let Some(buildings) = building_pos.get(&tile) {
            for (building_e, building_pos, building) in buildings {
                let dist = unit_t.translation.distance(*building_pos);

                let range = if unit.name.is_melee() {
                    (unit.name.world_size() + building.name.world_size().x) * 0.4
                } else {
                    unit.range(player) * RADIUS
                };

                if unit.name.can_attack() && building.color != unit.color && dist <= range {
                    unit.action = Action::Attack(*building_e);
                    return;
                }
            }
        }
    }

    // Units on buildings don't move
    if unit.on_building.is_some() {
        return;
    } else if enemy.has_boost(Boost::Frozen) {
        unit.action = Action::Idle;
        return;
    }

    let desired = (target_pos - unit_t.translation).normalize();

    // Prevent backwards walking to avoid flickering of blocked units
    let dot_prod = separation.dot(desired);
    if dot_prod < 0. {
        separation -= desired * dot_prod;
    }

    let movement = desired + 7. * separation.normalize_or_zero();

    let mut next_pos = unit_t.translation
        + movement.normalize()
            * unit.name.speed()
            * if player.has_boost(Boost::Run) {
                2.
            } else {
                1.
            }
            * if player.strategy == Strategy::March {
                1.5
            } else {
                1.
            }
            * settings.speed
            * time.delta_secs().min(CAPPED_DELTA_SECS_SPEED);

    let next_tile = Map::world_to_tile(&next_pos);

    if tile == next_tile || Map::is_walkable(next_tile) {
        // Check if the tile below is walkable. If not, restrict movement to the top part
        if !Map::is_walkable(TilePos::new(next_tile.x, next_tile.y + 1)) {
            let bottom_limit = Map::tile_to_world(next_tile).y - Map::TILE_SIZE as f32 * 0.5
                + unit.name.world_size() * 0.25;

            if next_pos.y < bottom_limit {
                next_pos.y = bottom_limit;
            }
        }

        // Change the direction the unit is facing
        unit_s.flip_x = if next_tile.x != tile.x {
            next_tile.x < tile.x
        } else {
            players.me.color != unit.color
        };

        unit_t.translation = next_pos;
        unit.action = Action::Run;
    } else {
        unit.action = Action::Idle;
    }
}

fn move_arrow(
    arrow_e: Entity,
    arrow: &mut Arrow,
    arrow_t: &mut Transform,
    arrow_s: &mut Sprite,
    apply_damage_msg: &mut MessageWriter<ApplyDamageMsg>,
    despawn_msg: &mut MessageWriter<DespawnMsg>,
    positions: &HashMap<TilePos, Vec<(Entity, PlayerColor, Vec3)>>,
    settings: &Settings,
    images: &Assets<Image>,
    time: &Time,
) {
    // Resolve arrow hitting an enemy
    arrow.traveled +=
        Arrow::SPEED * settings.speed * time.delta_secs().min(CAPPED_DELTA_SECS_SPEED);

    // Calculate progress (0.0 to 1.0)
    let progress = (arrow.traveled / arrow.total_distance).min(1.0);

    // Check if arrow reached destination
    if progress >= 1.0 {
        match arrow.projectile.mode() {
            ProjectileMode::Parabolic => {
                if let Some(image) = images.get(&arrow_s.image) {
                    // Hide the point to look as if the arrow is stuck in the ground
                    arrow_s.rect = Some(Rect {
                        min: Vec2::ZERO,
                        max: Vec2::new(image.width() as f32 * 0.65, image.height() as f32),
                    });

                    // Place ground arrows behind units and buildings
                    arrow_t.translation.z = BUILDINGS_Z - 0.1;
                }

                arrow.despawn_timer.tick(scale_duration(time.delta(), settings.speed));
                if arrow.despawn_timer.just_finished() {
                    despawn_msg.write(DespawnMsg(arrow_e));
                }
            },
            ProjectileMode::Straight => {
                despawn_msg.write(DespawnMsg(arrow_e));
            },
        }

        return;
    }

    // Linear interpolation between start and destination
    let mut pos = arrow.start.lerp(arrow.destination, progress);

    if arrow.projectile.mode() == ProjectileMode::Parabolic {
        // Parabolic arc: height = progress * (1 - progress) * 4 * arc_factor
        // Arc height is proportional to distance (20% of total distance as max height)
        let arc_height = progress * (1.0 - progress) * 4.0 * arrow.total_distance * 0.2;
        pos += Vec2::Y * arc_height;
    }

    // Set new position with arc
    arrow_t.translation = pos.extend(ARROW_Z);

    // Check if the arrow hit someone (in this or adjacent tiles)
    let tile = Map::world_to_tile(&arrow_t.translation);
    for tile in get_tiles_at_distance(&tile, 2) {
        if let Some(units) = positions.get(&tile) {
            for (other_e, other_color, other_pos) in units {
                if *other_color != arrow.color
                    && arrow_t.translation.distance(*other_pos) < RADIUS * 0.4
                {
                    apply_damage_msg.write(ApplyDamageMsg::new(*other_e, arrow.damage));
                    despawn_msg.write(DespawnMsg(arrow_e));
                    return;
                }
            }
        }
    }

    if arrow.projectile.mode() == ProjectileMode::Parabolic {
        // Calculate velocity direction for rotation (take a small step ahead to determine angle)
        let next_progress = ((arrow.traveled + 1.) / arrow.total_distance).min(1.);
        let next_horizontal = arrow.start.lerp(arrow.destination, next_progress);
        let next_arc_height =
            next_progress * (1. - next_progress) * 4.0 * arrow.total_distance * 0.2;
        let next_pos = next_horizontal + Vec2::Y * next_arc_height;

        let velocity = next_pos - arrow_t.translation.truncate();
        if velocity.length() > 0.01 {
            let angle = velocity.y.atan2(velocity.x);
            arrow_t.rotation = Quat::from_rotation_z(angle + arrow.projectile.angle());
        }
    }
}

pub fn apply_movement(
    mut unit_q: Query<(Entity, &mut Transform, &mut Sprite, &mut Unit)>,
    building_q: Query<(Entity, &Transform, &Building), Without<Unit>>,
    mut arrow_q: Query<
        (Entity, &mut Transform, &mut Sprite, &mut Arrow),
        (Without<Unit>, Without<Building>),
    >,
    mut apply_damage_msg: MessageWriter<ApplyDamageMsg>,
    mut despawn_msg: MessageWriter<DespawnMsg>,
    settings: Res<Settings>,
    map: Res<Map>,
    players: Res<Players>,
    images: Res<Assets<Image>>,
    time: Res<Time>,
) {
    // Build spatial hashmap: tile -> positions + unit
    let unit_pos: HashMap<TilePos, Vec<(Entity, Vec3, Unit)>> =
        unit_q.iter().fold(HashMap::new(), |mut acc, (e, t, _, u)| {
            let tile = Map::world_to_tile(&t.translation);
            acc.entry(tile).or_default().push((e, t.translation, *u));
            acc
        });

    let building_pos: HashMap<TilePos, Vec<(Entity, Vec3, Building)>> =
        building_q.iter().fold(HashMap::new(), |mut acc, (e, t, b)| {
            let tile = Map::world_to_tile(&t.translation);
            acc.entry(tile).or_default().push((e, t.translation, *b));
            acc
        });

    let any_pos: HashMap<TilePos, Vec<(Entity, PlayerColor, Vec3)>> = unit_pos
        .iter()
        .flat_map(|(t, u)| u.iter().map(|(e, p, u)| (*t, *e, u.color, *p)))
        .chain(
            building_pos.iter().flat_map(|(t, b)| b.iter().map(|(e, p, b)| (*t, *e, b.color, *p))),
        )
        .fold(HashMap::new(), |mut acc, (t, e, c, p)| {
            acc.entry(t).or_default().push((e, c, p));
            acc
        });

    // Move units
    for (unit_e, mut unit_t, mut unit_s, mut unit) in
        unit_q.iter_mut().filter(|(_, _, _, u)| matches!(u.action, Action::Idle | Action::Run))
    {
        move_unit(
            unit_e,
            &mut unit,
            &mut unit_t,
            &mut unit_s,
            &unit_pos,
            &building_pos,
            &settings,
            &map,
            &players,
            &time,
        );
    }

    // Move arrows
    for (arrow_e, mut arrow_t, mut arrow_s, mut arrow) in &mut arrow_q {
        move_arrow(
            arrow_e,
            &mut arrow,
            &mut arrow_t,
            &mut arrow_s,
            &mut apply_damage_msg,
            &mut despawn_msg,
            &any_pos,
            &settings,
            &images,
            &time,
        )
    }
}
