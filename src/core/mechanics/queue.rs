use crate::core::audio::PlayAudioMsg;
use crate::core::constants::MAX_QUEUE_LENGTH;
use crate::core::player::Players;
use crate::core::units::soldiers::SoldierName;
use bevy::prelude::*;
use bevy_renet::renet::ClientId;
use strum::IntoEnumIterator;

#[derive(Message)]
pub struct QueueSoldierMsg {
    pub id: ClientId,
    pub soldier: SoldierName,
}

impl QueueSoldierMsg {
    pub fn new(id: ClientId, soldier: SoldierName) -> Self {
        QueueSoldierMsg {
            id,
            soldier,
        }
    }
}

pub fn queue_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    players: Res<Players>,
    mut queue_soldier_msg: MessageWriter<QueueSoldierMsg>,
) {
    for soldier in SoldierName::iter() {
        if keyboard.just_pressed(soldier.key()) {
            queue_soldier_msg.write(QueueSoldierMsg::new(players.me.id, soldier));
        }
    }
}

pub fn queue_message(
    mut queue_soldier_msg: MessageReader<QueueSoldierMsg>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut players: ResMut<Players>,
) {
    for message in queue_soldier_msg.read() {
        let player = players.get_mut(message.id);

        if player.queue.len() < MAX_QUEUE_LENGTH {
            player.queue.push_back(message.soldier);
            if player.is_human() {
                play_audio_msg.write(PlayAudioMsg::new("button"));
            }
        } else if player.is_human() {
            play_audio_msg.write(PlayAudioMsg::new("error"));
        }
    }
}

pub fn resolve_queue(mut players: ResMut<Players>, time: Res<Time>) {}
