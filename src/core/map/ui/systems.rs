use crate::core::assets::WorldAssets;
use crate::core::constants::MAX_QUEUE_LENGTH;
use crate::core::map::systems::{BgAnimCmp, MapCmp, SpeedCmp};
use crate::core::menu::utils::add_text;
use crate::core::player::Players;
use crate::core::settings::Settings;
use crate::core::states::GameState;
use crate::core::units::soldiers::SoldierName;
use crate::utils::NameFromEnum;
use bevy::prelude::*;
use bevy_tweening::{PlaybackState, TweenAnim};

#[derive(Component)]
pub struct UiCmp;

#[derive(Component)]
pub struct QueueButtonCmp {
    pub position: usize,
    pub soldier: SoldierName,
}

impl QueueButtonCmp {
    pub fn new(position: usize, soldier: SoldierName) -> Self {
        Self {
            position,
            soldier,
        }
    }
}

pub fn draw_ui(
    mut commands: Commands,
    settings: Res<Settings>,
    window: Single<&Window>,
    assets: Local<WorldAssets>,
) {
    // Draw queue
    commands
        .spawn((
            Node {
                bottom: Val::Percent(5.),
                width: Val::Percent(100.),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            UiCmp,
            MapCmp,
        ))
        .with_children(|parent| {
            for i in 0..MAX_QUEUE_LENGTH {
                parent
                    .spawn((
                        Node {
                            width: Val::Percent(7.),
                            margin: UiRect::ZERO.with_left(Val::Percent(1.)),
                            ..default()
                        },
                        ImageNode::new(assets.image("Blue-Warrior")),
                        QueueButtonCmp::new(i, SoldierName::default()),
                        Visibility::Hidden,
                    ))
                    .observe(
                        |event: On<Pointer<Click>>,
                         btn_q: Query<&QueueButtonCmp>,
                         mut players: ResMut<Players>| {
                            // Remove soldier from queue if right-clicked
                            if event.button == PointerButton::Secondary {
                                if let Ok(button) = btn_q.get(event.entity) {
                                    players.me.queue.remove(button.position);
                                }
                            }
                        },
                    );
            }
        });

    // Draw speed indicator
    commands.spawn((
        Node {
            bottom: Val::Px(10.),
            left: Val::Px(10.),
            position_type: PositionType::Absolute,
            ..default()
        },
        add_text(format!("{}x", settings.speed), "medium", 10., &assets, &window),
        SpeedCmp,
        MapCmp,
    ));
}

pub fn update_ui(
    mut queue_q: Query<(&mut Visibility, &mut ImageNode, &mut QueueButtonCmp)>,
    mut anim_q: Query<(&mut TweenAnim, Option<&BgAnimCmp>)>,
    mut speed_q: Single<&mut Text, With<SpeedCmp>>,
    settings: Res<Settings>,
    players: Res<Players>,
    game_state: Res<State<GameState>>,
    assets: Local<WorldAssets>,
) {
    for (mut visibility, mut image, button) in &mut queue_q {
        if let Some(soldier) = players.me.queue.get(button.position) {
            *visibility = Visibility::Inherited;

            image.image = assets.image(format!(
                "{}-{}",
                players.me.color.to_name(),
                button.soldier.to_name()
            ));
        } else {
            *visibility = Visibility::Hidden;
        }
    }

    // Play/pause tween animations
    anim_q.iter_mut().for_each(|(mut t, a)| match game_state.get() {
        GameState::Playing => {
            t.playback_state = PlaybackState::Playing;
            if a.is_none() {
                // Ignore background animations (e.g., water foam) from speed changes
                t.speed = settings.speed as f64;
            }
        },
        _ => t.playback_state = PlaybackState::Paused,
    });

    // Update speed indicator
    speed_q.as_mut().0 = format!(
        "{}x{}",
        settings.speed,
        match game_state.get() {
            GameState::Playing => "",
            _ => " - paused",
        },
    );
}
