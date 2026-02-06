use crate::core::assets::WorldAssets;
use crate::core::constants::*;
use crate::core::map::map::Lane;
use crate::core::map::systems::MapCmp;
use crate::core::map::ui::systems::UnitInfoCmp;
use crate::core::map::utils::SpriteFrameLens;
use crate::core::mechanics::combat::{Arrow, Projectile};
use crate::core::mechanics::effects::EffectMsg;
#[cfg(not(target_arch = "wasm32"))]
use crate::core::multiplayer::EntityMap;
use crate::core::player::Players;
use crate::core::settings::PlayerColor;
use crate::core::states::GameState;
use crate::core::units::buildings::{Building, BuildingName};
use crate::core::units::units::{Action, Unit, UnitName};
use crate::core::utils::cursor;
use crate::utils::NameFromEnum;
use bevy::color::palettes::css::{BLACK, LIME};
use bevy::color::Color;
use bevy::ecs::children;
use bevy::math::Vec3;
use bevy::prelude::*;
use bevy::window::SystemCursorIcon;
use bevy_tweening::{RepeatCount, Tween, TweenAnim};
use std::f32::consts::FRAC_PI_4;
use std::time::Duration;

#[derive(Component)]
pub struct HealthWrapperCmp;

#[derive(Component)]
pub struct HealthCmp;

#[derive(Message)]
pub struct SpawnBuildingMsg {
    pub color: PlayerColor,
    pub building: BuildingName,
    pub position: Vec2,
    pub is_base: bool,
    pub health: f32,
    pub with_units: bool,
    pub dust_effect: bool,
    pub entity: Option<Entity>,
}

#[derive(Message)]
pub struct SpawnUnitMsg {
    pub color: PlayerColor,
    pub unit: UnitName,
    pub position: Option<Vec2>,
    pub on_building: Option<Entity>,
    pub lane: Option<Lane>,
    pub dust_effect: bool,
    pub entity: Option<Entity>,
}

impl SpawnUnitMsg {
    pub fn new(color: PlayerColor, unit: UnitName) -> Self {
        Self {
            color,
            unit,
            position: None,
            on_building: None,
            lane: None,
            dust_effect: false,
            entity: None,
        }
    }
}

#[derive(Message)]
pub struct SpawnArrowMsg {
    pub color: PlayerColor,
    pub projectile: Projectile,
    pub damage: f32,
    pub start: Vec2,
    pub destination: Vec2,
    pub entity: Option<Entity>,
}

#[derive(Message)]
pub struct DespawnMsg(pub Entity);

pub fn spawn_building_message(
    mut commands: Commands,
    #[cfg(not(target_arch = "wasm32"))] mut entity_map: ResMut<EntityMap>,
    mut spawn_building_msg: MessageReader<SpawnBuildingMsg>,
    mut effct_msg: MessageWriter<EffectMsg>,
    mut spawn_unit_msg: MessageWriter<SpawnUnitMsg>,
    assets: Res<WorldAssets>,
) {
    for msg in spawn_building_msg.read() {
        let size = msg.building.size();

        let id = commands
            .spawn((
                Sprite {
                    image: assets.image(format!(
                        "{}-{}",
                        msg.color.to_name(),
                        msg.building.to_name()
                    )),
                    custom_size: Some(size),
                    ..default()
                },
                Transform {
                    translation: msg.position.extend(BUILDINGS_Z),
                    scale: Vec3::splat(BUILDING_SCALE),
                    ..default()
                },
                Building::new(msg.building, msg.color, msg.is_base, msg.health),
                MapCmp,
                children![(
                    Sprite {
                        color: Color::from(BLACK),
                        custom_size: Some(Vec2::new(0.5 * size.x, 15.)),
                        ..default()
                    },
                    Transform::from_xyz(0., size.y * 0.5, EFFECT_Z - 0.2),
                    Visibility::Hidden,
                    HealthWrapperCmp,
                    children![(
                        Sprite {
                            color: Color::from(LIME),
                            custom_size: Some(Vec2::new(0.49 * size.x, 13.)),
                            ..default()
                        },
                        Transform::from_xyz(0., 0., EFFECT_Z - 0.1),
                        HealthCmp,
                    )],
                )],
            ))
            .id();

        if msg.dust_effect {
            effct_msg.write(EffectMsg::dust(id));
        }

        if msg.with_units {
            for pos in msg.building.units() {
                spawn_unit_msg.write(SpawnUnitMsg {
                    color: msg.color,
                    unit: UnitName::Archer,
                    position: Some(msg.position + pos),
                    on_building: Some(id),
                    lane: None,
                    dust_effect: false,
                    entity: None,
                });
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(entity) = msg.entity {
            entity_map.insert(entity, id);
        }
    }
}

pub fn spawn_unit_message(
    mut commands: Commands,
    building_q: Query<(&Transform, &Building)>,
    players: Res<Players>,
    #[cfg(not(target_arch = "wasm32"))] mut entity_map: ResMut<EntityMap>,
    mut effect_msg: MessageWriter<EffectMsg>,
    mut spawn_unit_msg: MessageReader<SpawnUnitMsg>,
    assets: Res<WorldAssets>,
) {
    for msg in spawn_unit_msg.read() {
        let unit = msg.unit;
        let action = Action::default();

        let atlas = assets.atlas(format!(
            "{}-{}-{}",
            msg.color.to_name(),
            unit.to_name(),
            action.to_name()
        ));

        // Determine the spawning translation
        // If not provided, use the default position at the door of the base
        let translation = if let Some(pos) = msg.position {
            Some(pos.extend(UNITS_Z))
        } else {
            building_q
                .iter()
                .find(|(_, b)| b.color == msg.color && b.is_base)
                .map(|(t, _)| Vec3::new(t.translation.x, t.translation.y - 70., UNITS_Z))
        };

        if let Some(translation) = translation {
            let id = commands
                .spawn((
                    Sprite {
                        image: atlas.image,
                        texture_atlas: Some(atlas.atlas),
                        custom_size: Some(Vec2::splat(unit.size())),
                        flip_x: players.me.color != msg.color,
                        ..default()
                    },
                    Transform {
                        translation,
                        scale: Vec3::splat(UNIT_SCALE),
                        ..default()
                    },
                    TweenAnim::new(
                        Tween::new(
                            EaseFunction::Linear,
                            Duration::from_millis(FRAME_RATE * unit.frames(action) as u64),
                            SpriteFrameLens(atlas.last_index),
                        )
                        .with_repeat_count(RepeatCount::Infinite),
                    ),
                    Pickable::default(),
                    Unit::new(unit, players.get_by_color(msg.color), msg.lane, msg.on_building),
                    MapCmp,
                    children![(
                        Sprite {
                            color: Color::from(BLACK),
                            custom_size: Some(4. + HEALTH_SIZE),
                            ..default()
                        },
                        Transform::from_xyz(0., unit.world_size() * 0.7, 0.1),
                        Visibility::Hidden,
                        HealthWrapperCmp,
                        children![(
                            Sprite {
                                color: Color::from(LIME),
                                custom_size: Some(HEALTH_SIZE),
                                ..default()
                            },
                            Transform::from_xyz(0., 0., 0.2),
                            HealthCmp,
                        )],
                    )],
                ))
                .observe(cursor::<Over>(SystemCursorIcon::Pointer))
                .observe(cursor::<Out>(SystemCursorIcon::Default))
                .observe(
                    move |_: On<Pointer<Click>>,
                          mut info_q: Query<(&mut Visibility, &UnitInfoCmp)>,
                          game_state: Res<State<GameState>>,
                          mut next_game_state: ResMut<NextState<GameState>>| {
                        if matches!(
                            game_state.get(),
                            GameState::Playing | GameState::Paused | GameState::UnitInfo
                        ) {
                            for (mut v, i) in info_q.iter_mut() {
                                *v = if **i == unit {
                                    Visibility::Inherited
                                } else {
                                    Visibility::Hidden
                                }
                            }
                            next_game_state.set(GameState::UnitInfo);
                        }
                    },
                )
                .id();

            if msg.dust_effect {
                effect_msg.write(EffectMsg::dust(id));
            }

            #[cfg(not(target_arch = "wasm32"))]
            if let Some(entity) = msg.entity {
                entity_map.insert(entity, id);
            }
        }
    }
}

pub fn spawn_arrow_message(
    mut commands: Commands,
    #[cfg(not(target_arch = "wasm32"))] mut entity_map: ResMut<EntityMap>,
    mut spawn_arrow_msg: MessageReader<SpawnArrowMsg>,
    assets: Res<WorldAssets>,
) {
    for msg in spawn_arrow_msg.read() {
        let sprite = if msg.projectile.animation() {
            let atlas = assets.atlas(msg.projectile.to_lowername());
            Sprite {
                image: atlas.image,
                texture_atlas: Some(atlas.atlas),
                ..default()
            }
        } else {
            Sprite {
                image: assets.image(msg.projectile.to_lowername()),
                ..default()
            }
        };

        let id = commands
            .spawn((
                sprite,
                Transform {
                    translation: msg.start.extend(ARROW_Z),
                    rotation: Quat::from_rotation_z(FRAC_PI_4 + msg.projectile.angle()),
                    scale: Vec3::splat(UNIT_SCALE),
                },
                Arrow::new(msg.color, msg.projectile, msg.damage, msg.start, msg.destination),
                MapCmp,
            ))
            .id();

        if msg.projectile.animation() {
            let atlas = assets.atlas(msg.projectile.to_lowername());
            commands.entity(id).insert(TweenAnim::new(
                Tween::new(
                    EaseFunction::Linear,
                    Duration::from_millis(FRAME_RATE * (atlas.last_index + 1) as u64),
                    SpriteFrameLens(atlas.last_index),
                )
                .with_repeat_count(RepeatCount::Infinite),
            ));
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(entity) = msg.entity {
            entity_map.insert(entity, id);
        }
    }
}

pub fn despawn_message(
    mut commands: Commands,
    unit_q: Query<(Entity, &Unit)>,
    mut despawn_message: MessageReader<DespawnMsg>,
) {
    for msg in despawn_message.read() {
        // Try since there can be multiple messages to despawn the same entity
        commands.entity(msg.0).try_despawn();

        // Despawn any units on top of this building
        for (unit_e, unit) in unit_q.iter() {
            if unit.on_building == Some(msg.0) {
                commands.entity(unit_e).despawn();
            }
        }
    }
}
