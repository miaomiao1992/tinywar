use crate::core::assets::WorldAssets;
use crate::core::audio::PlayAudioMsg;
use crate::core::constants::{BUILDING_SCALE, EFFECT_Z, FRAME_RATE, UNIT_SCALE};
use crate::core::map::systems::MapCmp;
use crate::core::map::utils::SpriteFrameLens;
use crate::core::menu::systems::Host;
use crate::core::units::buildings::Building;
use crate::core::units::units::Unit;
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use bevy_tweening::{CycleCompletedEvent, Delay, Tween, TweenAnim};
use rand::{rng, Rng};
use serde::{Deserialize, Serialize};
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use {
    crate::core::multiplayer::EntityMap,
    crate::core::network::{ServerMessage, ServerSendMsg},
};

#[derive(Component)]
pub struct EffectCmp;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Effect {
    Dust,
    Explosion,
}

#[derive(Message)]
pub struct EffectMsg {
    pub effect: Effect,
    pub entity: Entity,
}

impl EffectMsg {
    pub fn dust(entity: Entity) -> Self {
        Self {
            effect: Effect::Dust,
            entity,
        }
    }

    pub fn explosion(entity: Entity) -> Self {
        Self {
            effect: Effect::Explosion,
            entity,
        }
    }
}

#[derive(Message, Clone)]
pub struct DeferredEffectMsg {
    pub effect: Effect,
    pub entity: Entity,
}

pub fn effect_message(
    mut commands: Commands,
    building_q: Query<(&Transform, &Building)>,
    unit_q: Query<(&Transform, &Unit)>,
    host: Option<Res<Host>>,
    #[cfg(not(target_arch = "wasm32"))] entity_map: Res<EntityMap>,
    #[cfg(not(target_arch = "wasm32"))] mut server_send_msg: MessageWriter<ServerSendMsg>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut effect_msg: MessageReader<EffectMsg>,
    mut deferred_msg: MessageWriter<DeferredEffectMsg>,
    assets: Res<WorldAssets>,
) {
    for msg in effect_msg.read() {
        #[cfg(not(target_arch = "wasm32"))]
        let entity = if host.is_some() {
            server_send_msg.write(ServerSendMsg::new(
                ServerMessage::Effect {
                    effect: msg.effect,
                    entity: msg.entity,
                },
                None,
            ));
            msg.entity
        } else {
            *entity_map.get_by_left(&msg.entity).unwrap_or(&msg.entity)
        };

        let (translation, size, particles, radius, scale) =
            if let Ok((building_t, building)) = building_q.get(entity) {
                (building_t.translation, building.name.size() * BUILDING_SCALE, 20, 0.3, 1.0)
            } else if let Ok((unit_t, unit)) = unit_q.get(entity) {
                (unit_t.translation, Vec2::splat(unit.name.size()) * UNIT_SCALE, 3, 0.1, 0.5)
            } else {
                // Entity wasn't spawned yet -> defer message for next frame
                deferred_msg.write(DeferredEffectMsg {
                    effect: msg.effect,
                    entity: msg.entity,
                });
                return;
            };

        if msg.effect == Effect::Explosion {
            play_audio_msg.write(PlayAudioMsg::new("explosion"));
        }

        let mut rng = rng();

        for _ in 0..particles {
            let atlas =
                assets.atlas(format!("{}{}", msg.effect.to_lowername(), rng.random_range(1..3)));

            commands.spawn((
                Sprite {
                    image: atlas.image,
                    texture_atlas: Some(atlas.atlas),
                    ..default()
                },
                Transform {
                    translation: (translation.truncate()
                        + Vec2::new(
                            rng.random_range(-radius * size.x..radius * size.x),
                            rng.random_range(-radius * size.y..radius * size.y),
                        ))
                    .extend(EFFECT_Z),
                    scale: Vec3::splat(rng.random_range(scale..scale + 0.5)),
                    ..default()
                },
                TweenAnim::new(
                    Delay::new(Duration::from_millis(match msg.effect {
                        Effect::Dust => 1,
                        Effect::Explosion => rng.random_range(1..1500),
                    }))
                    .then(
                        Tween::new(
                            EaseFunction::Linear,
                            Duration::from_millis(FRAME_RATE * atlas.last_index as u64),
                            SpriteFrameLens(atlas.last_index),
                        )
                        .with_cycle_completed_event(true),
                    ),
                ),
                EffectCmp,
                MapCmp,
            ));
        }
    }
}

pub fn deferred_message(
    mut deferred_msg: MessageReader<DeferredEffectMsg>,
    mut effect_msg: MessageWriter<EffectMsg>,
) {
    for msg in deferred_msg.read() {
        effect_msg.write(EffectMsg {
            effect: msg.effect,
            entity: msg.entity,
        });
    }
}

pub fn despawn_effects(
    mut commands: Commands,
    explosion_q: Query<Entity, With<EffectCmp>>,
    mut cycle_completed_msg: MessageReader<CycleCompletedEvent>,
) {
    for msg in cycle_completed_msg.read() {
        if let Ok(entity) = explosion_q.get(msg.anim_entity) {
            commands.entity(entity).despawn();
        }
    }
}
