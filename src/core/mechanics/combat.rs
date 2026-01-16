use crate::core::assets::WorldAssets;
use crate::core::constants::{
    ARROW_ARC_HEIGHT, ARROW_MAX_DISTANCE, ARROW_SPEED, CAPPED_DELTA_SECS_SPEED,
};
use crate::core::map::systems::MapCmp;
use crate::core::mechanics::spawn::DespawnMsg;
use crate::core::settings::Settings;
use crate::core::units::units::{Action, Unit, UnitName};
use bevy::prelude::*;
use bevy_tweening::CycleCompletedEvent;
use std::f32::consts::{FRAC_PI_4, PI};

#[derive(Component)]
pub struct Arrow {
    pub target: Entity,
    pub distance: f32,
}

impl Arrow {
    pub fn new(target: Entity) -> Self {
        Arrow {
            target,
            distance: 0.,
        }
    }
}

pub fn apply_damage(
    mut commands: Commands,
    mut unit_q: Query<(Entity, &Transform, &mut Unit)>,
    mut cycle_completed_msg: MessageReader<CycleCompletedEvent>,
    mut despawn_msg: MessageWriter<DespawnMsg>,
    assets: Local<WorldAssets>,
) {
    // Apply damage after the attacking animation finished
    for msg in cycle_completed_msg.read() {
        let (unit, unit_t) = if let Ok((_, unit_t, unit)) = unit_q.get(msg.anim_entity) {
            (*unit, unit_t.clone())
        } else {
            continue;
        };

        match unit.action {
            Action::Attack(e) | Action::Heal(e) => {
                let action_finished =
                    unit_q.get_mut(e).map_or(true, |(target_e, _, mut target)| {
                        if unit.name == UnitName::Archer {
                            // Archers don't apply damage but spawn arrows at the end of the animation
                            commands.spawn((
                                Sprite {
                                    image: assets.image("arrow"),
                                    ..default()
                                },
                                Transform {
                                    translation: unit_t.translation,
                                    rotation: Quat::from_rotation_z(FRAC_PI_4),
                                    scale: unit_t.scale,
                                },
                                Arrow::new(target_e),
                                MapCmp,
                            ));

                            return false;
                        }

                        target.health = (target.health - unit.name.damage()).max(0.);
                        if target.health == 0. {
                            despawn_msg.write(DespawnMsg(target_e));
                        }

                        target.health <= 0. || target.health >= target.name.health()
                    });

                if action_finished {
                    if let Ok((_, _, mut unit)) = unit_q.get_mut(msg.anim_entity) {
                        unit.action = Action::Idle;
                    }
                }
            },
            _ => (),
        }
    }
}

pub fn move_arrows(
    mut arrow_q: Query<(Entity, &mut Transform, &mut Arrow)>,
    mut unit_q: Query<(Entity, &mut Unit)>,
    mut despawn_msg: MessageWriter<DespawnMsg>,
    settings: Res<Settings>,
    time: Res<Time>,
) {
    for (arrow_e, mut arrow_t, mut arrow) in &mut arrow_q {
        // Calculate direction based on current rotation
        let direction = arrow_t.rotation * Vec3::X;

        // Move arrow along its direction
        let base_movement = direction.normalize()
            * ARROW_SPEED
            * settings.speed
            * time.delta_secs().min(CAPPED_DELTA_SECS_SPEED);

        // Add vertical arc component
        let progress = (arrow.distance / ARROW_MAX_DISTANCE).min(1.0);
        let arc_velocity = (progress * PI).cos() * ARROW_ARC_HEIGHT * PI / ARROW_MAX_DISTANCE;
        let arc_movement = Vec3::Y * arc_velocity * time.delta_secs();

        arrow_t.translation += base_movement + arc_movement;

        // Update rotation based on velocity
        let velocity = base_movement + arc_movement;
        let traveled = velocity.length();
        if traveled > 0.01 {
            let angle = velocity.y.atan2(velocity.x);
            arrow_t.rotation = Quat::from_rotation_z(angle);
        }

        arrow.distance += traveled;
        if arrow.distance > ARROW_MAX_DISTANCE {
            despawn_msg.write(DespawnMsg(arrow_e));
        }
    }
}
