use crate::core::audio::PlayAudioMsg;
use crate::core::constants::{MAX_GAME_SPEED, MIN_GAME_SPEED};
use crate::core::map::ui::systems::UiCmp;
use crate::core::mechanics::effects::EffectCmp;
use crate::core::mechanics::queue::QueueUnitMsg;
use crate::core::menu::systems::{Host, StartNewGameMsg};
use crate::core::menu::utils::TextSize;
use crate::core::player::{PlayerDirection, Players, Side, Strategy};
use crate::core::settings::Settings;
use crate::core::states::{AppState, GameState};
use crate::core::units::units::{Action, Unit, UnitName};
use crate::utils::scale_duration;
use bevy::prelude::*;
use bevy::window::WindowResized;
#[cfg(not(target_arch = "wasm32"))]
use bevy_renet::RenetServer;
use bevy_tweening::{PlaybackState, TweenAnim};
use strum::IntoEnumIterator;

pub fn on_resize_message(
    mut resize_msg: MessageReader<WindowResized>,
    mut text: Query<(&mut TextFont, &TextSize)>,
) {
    for window in resize_msg.read() {
        for (mut text, size) in text.iter_mut() {
            text.font_size = size.0 * window.height / 460.
        }
    }
}

pub fn check_keys_menu(
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    #[cfg(not(target_arch = "wasm32"))] server: Option<Res<RenetServer>>,
    mut start_new_game_msg: MessageWriter<StartNewGameMsg>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_released(KeyCode::Escape) {
        match app_state.get() {
            AppState::SinglePlayerMenu | AppState::MultiPlayerMenu | AppState::Settings => {
                next_app_state.set(AppState::MainMenu)
            },
            AppState::Lobby | AppState::ConnectedLobby => {
                next_app_state.set(AppState::MultiPlayerMenu)
            },
            AppState::Game => match game_state.get() {
                GameState::Playing | GameState::Paused => next_game_state.set(GameState::GameMenu),
                GameState::UnitInfo | GameState::GameMenu => {
                    next_game_state.set(GameState::Playing)
                },
                GameState::EndGame => next_app_state.set(AppState::MainMenu),
                GameState::Settings => next_game_state.set(GameState::GameMenu),
                _ => (),
            },
            _ => (),
        }
    }

    if keyboard.just_released(KeyCode::KeyH) {
        if matches!(game_state.get(), GameState::Playing | GameState::Paused | GameState::EndGame) {
            next_game_state.set(GameState::UnitInfo);
        } else if *game_state.get() == GameState::UnitInfo {
            next_game_state.set(GameState::Playing);
        }
    }

    if keyboard.just_released(KeyCode::Enter) {
        match app_state.get() {
            AppState::MainMenu => next_app_state.set(AppState::SinglePlayerMenu),
            AppState::SinglePlayerMenu => {
                start_new_game_msg.write(StartNewGameMsg);
            },
            AppState::MultiPlayerMenu => next_app_state.set(AppState::Lobby),
            #[cfg(not(target_arch = "wasm32"))]
            AppState::ConnectedLobby if server.is_some() => {
                start_new_game_msg.write(StartNewGameMsg);
            },
            AppState::Settings => next_app_state.set(AppState::MainMenu),
            AppState::Game if *game_state.get() == GameState::EndGame => {
                next_app_state.set(AppState::MainMenu)
            },
            _ => (),
        }
    }
}

pub fn check_keys_game(
    keyboard: Res<ButtonInput<KeyCode>>,
    host: Option<Res<Host>>,
    mut settings: ResMut<Settings>,
    game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_released(KeyCode::Space) {
        match game_state.get() {
            GameState::Playing => next_game_state.set(GameState::Paused),
            GameState::Paused => next_game_state.set(GameState::Playing),
            _ => (),
        }
    } else if host.is_some() && keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
    {
        if keyboard.just_released(KeyCode::ArrowRight) {
            settings.speed = (settings.speed * 2.).min(MAX_GAME_SPEED);
        } else if keyboard.just_released(KeyCode::ArrowLeft) {
            settings.speed = (settings.speed * 0.5).max(MIN_GAME_SPEED);
        }
    }
}

pub fn check_keys_playing_game(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut players: ResMut<Players>,
    mut queue_unit_msg: MessageWriter<QueueUnitMsg>,
    mut play_audio_msg: MessageWriter<PlayAudioMsg>,
    mut pressed: Local<bool>,
) {
    // Change direction
    let mid_key = if players.me.side == Side::Left {
        KeyCode::ArrowRight
    } else {
        KeyCode::ArrowLeft
    };

    let any_key = if players.me.side == Side::Left {
        KeyCode::ArrowLeft
    } else {
        KeyCode::ArrowRight
    };

    if !keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
        let mut new_direction = None;

        if keyboard.just_released(any_key) {
            new_direction = Some(PlayerDirection::Any);
        } else if keyboard.just_released(mid_key) && !*pressed {
            new_direction = Some(if keyboard.pressed(KeyCode::ArrowUp) {
                *pressed = true;
                PlayerDirection::TopMid
            } else if keyboard.pressed(KeyCode::ArrowDown) {
                *pressed = true;
                PlayerDirection::MidBot
            } else {
                PlayerDirection::Mid
            });
        } else if keyboard.just_released(KeyCode::ArrowUp) && !*pressed {
            new_direction = Some(if keyboard.pressed(mid_key) {
                *pressed = true;
                PlayerDirection::TopMid
            } else if keyboard.pressed(KeyCode::ArrowDown) {
                *pressed = true;
                PlayerDirection::TopBot
            } else {
                PlayerDirection::Top
            });
        } else if keyboard.just_released(KeyCode::ArrowDown) && !*pressed {
            new_direction = Some(if keyboard.pressed(mid_key) {
                *pressed = true;
                PlayerDirection::MidBot
            } else if keyboard.pressed(KeyCode::ArrowUp) {
                *pressed = true;
                PlayerDirection::TopBot
            } else {
                PlayerDirection::Bot
            });
        }

        if let Some(direction) = new_direction {
            if players.me.direction != direction {
                play_audio_msg.write(PlayAudioMsg::new("click"));
                players.me.direction = direction;
            }
        } else if !keyboard.any_pressed([KeyCode::ArrowUp, mid_key, KeyCode::ArrowDown]) {
            *pressed = false;
        }
    }

    // Change strategy
    for strategy in Strategy::iter() {
        if keyboard.just_released(strategy.key()) && strategy != players.me.strategy {
            if players.me.strategy_timer.is_finished() {
                play_audio_msg.write(PlayAudioMsg::new("click"));
                players.me.strategy = strategy;
                players.me.strategy_timer.reset();
            } else {
                play_audio_msg.write(PlayAudioMsg::new("error"));
            }
        }
    }

    // Queue units
    for unit in UnitName::iter() {
        if keyboard.just_released(unit.key()) && players.me.can_queue(unit) {
            queue_unit_msg.write(QueueUnitMsg::new(players.me.id, unit));
            play_audio_msg.write(PlayAudioMsg::new("button"));
        }
    }
}

pub fn update_animations(
    mut anim_q: Query<(&mut TweenAnim, Option<&Unit>, Option<&EffectCmp>), Without<UiCmp>>,
    settings: Res<Settings>,
    players: Res<Players>,
    game_state: Res<State<GameState>>,
) {
    // Play/pause tween animations
    anim_q.iter_mut().for_each(|(mut tween, unit, explosion)| {
        tween.speed = settings.speed as f64;

        // Increase the attack animation of units in berserk mode
        if let Some(unit) = unit {
            let player = players.get_by_color(unit.color);

            if player.strategy == Strategy::Berserk && matches!(unit.action, Action::Attack(_)) {
                tween.speed *= 1.3
            }
        }

        // Skip pause for explosions
        if explosion.is_none() {
            tween.playback_state = match game_state.get() {
                GameState::Playing => PlaybackState::Playing,
                _ => PlaybackState::Paused,
            }
        }
    });
}

pub fn update_strategy_timer(
    settings: Res<Settings>,
    mut players: ResMut<Players>,
    time: Res<Time>,
) {
    players.me.strategy_timer.tick(scale_duration(time.delta(), settings.speed));
}
