use crate::core::assets::WorldAssets;
use crate::core::constants::MAP_Z;
use crate::core::map::map::Map;
use crate::core::map::utils::TileTextureLens;
use crate::core::settings::Settings;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_tweening::{Delay, RepeatCount, Tween, TweenAnim};
use rand::{rng, Rng};
use std::time::Duration;

#[derive(Component)]
pub struct MapCmp;

#[derive(Component)]
pub struct SpeedCmp;

#[derive(Component)]
pub struct BgAnimCmp;

pub fn draw_map(mut commands: Commands, settings: Res<Settings>, assets: Local<WorldAssets>) {
    let mut rng = rng();

    // Draw map
    let map = Map::new(&settings.map_size, &assets);
    for (z, layer) in map.layers.iter().enumerate() {
        let mut tile_storage = TileStorage::empty(map.size);
        let entity = commands.spawn_empty().id();

        for x in 0..map.size.x {
            for y in 0..map.size.y {
                let v = layer.grid[y as usize][x as usize];

                // u32::MAX used as marker to skip the tile
                if v != u32::MAX {
                    let tile_pos = TilePos::new(x, y);
                    let tile_entity = commands
                        .spawn((
                            TileBundle {
                                position: tile_pos,
                                tilemap_id: TilemapId(entity),
                                texture_index: TileTextureIndex(v),
                                ..default()
                            },
                            MapCmp,
                        ))
                        .id();

                    // Add animation to the tiles. Start at different frames for realistic effect
                    if let Some(last_idx) = layer.animation {
                        commands.entity(tile_entity).insert((
                            TweenAnim::new(
                                Delay::new(Duration::from_millis(rng.random_range(1..1000))).then(
                                    Tween::new(
                                        EaseFunction::Linear,
                                        Duration::from_millis(1250),
                                        TileTextureLens(last_idx),
                                    )
                                    .with_repeat_count(RepeatCount::Infinite),
                                ),
                            ),
                            BgAnimCmp,
                        ));
                    }

                    tile_storage.set(&tile_pos, tile_entity);
                }
            }
        }

        commands.entity(entity).insert(TilemapBundle {
            grid_size: map.grid_size,
            map_type: map.map_type,
            size: map.size,
            storage: tile_storage,
            texture: TilemapTexture::Single(layer.texture.clone()),
            tile_size: layer.tile_size,
            anchor: TilemapAnchor::Center,
            transform: Transform::from_xyz(0., 0., MAP_Z + 0.1 * z as f32),
            ..default()
        });
    }
}
