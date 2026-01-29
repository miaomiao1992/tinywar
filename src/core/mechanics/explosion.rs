use std::time::Duration;
use bevy::prelude::*;
use bevy_tweening::{Delay, Tween, TweenAnim};
use rand::rng;
use crate::core::audio::PlayAudioMsg;
use crate::core::constants::{EXPLOSION_Z, FRAME_RATE};
use crate::core::map::utils::SpriteFrameLens;
use crate::core::menu::systems::Host;
use crate::core::multiplayer::EntityMap;
use crate::core::network::{ServerMessage, ServerSendMsg};
use crate::core::units::buildings::Building;

#[derive(Message)]
pub struct ExplosionMsg(pub Entity);

pub fn explosion_message(
    mut commands: Commands,
    buildings: Query<&Building>,
    host: Option<Res<Host>>,
    entity_map: Res<EntityMap>,
    mut server_send_msg: MessageWriter<ServerSendMsg>,
    mut explosion_msg: MessageReader<ExplosionMsg>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    assets: Res<AssetServer>,
) {
    for ExplosionMsg(entity) in explosion_msg.read() {
        if host.is_some() {
            let entity = entity_map.get_by_left(entity).unwrap_or(entity);
            server_send_msg.write(ServerSendMsg::new(ServerMessage::Explosion(*entity), None));
        }

        let mut rng = rng();
        play_audio_msg.write(PlayAudioMsg::new("explosion"));

        // let size = building.name.size();
        //                     for _ in 0..7 {
        //                         let atlas =
        //                             assets.atlas(format!("explosion{}", rng.random_range(1..3)));
        //
        //                         parent.spawn((
        //                             Sprite {
        //                                 image: atlas.image,
        //                                 texture_atlas: Some(atlas.atlas),
        //                                 ..default()
        //                             },
        //                             Transform {
        //                                 translation: Vec3::new(
        //                                     rng.random_range(-0.4 * size.x..0.4 * size.x),
        //                                     rng.random_range(-0.3 * size.y..0.3 * size.y),
        //                                     EXPLOSION_Z,
        //                                 ),
        //                                 scale: Vec3::splat(rng.random_range(1.0..1.5)),
        //                                 ..default()
        //                             },
        //                             TweenAnim::new(
        //                                 Delay::new(Duration::from_millis(
        //                                     rng.random_range(1..1000),
        //                                 ))
        //                                 .then(Tween::new(
        //                                     EaseFunction::Linear,
        //                                     Duration::from_millis(
        //                                         FRAME_RATE * atlas.last_index as u64,
        //                                     ),
        //                                     SpriteFrameLens(atlas.last_index),
        //                                 )),
        //                             ),
        //                         ));
        //                     }
        //                 });
    }
}
