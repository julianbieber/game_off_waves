use bevy::prelude::*;

use crate::{
    demo::{
        player::{PlayerStats, StatIncreases},
        weapons::{WeaponSlots, WeaponType},
    },
    screens::Screen,
    theme::widget::{button, label},
};

pub struct UpgradePlugin;

#[derive(Resource)]
pub struct AvailableUpgrades {
    pub stats: u32,
    pub gold: u32,
    pub current_shop: Option<(WeaponType, u32)>,
}

impl Plugin for UpgradePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Shop), setup_upgrade_ui);
        app.add_systems(
            Update,
            (
                update_upgrade_text,
                update_stat_button_text,
                update_weapon_button_texts,
            ),
        );
        app.insert_resource(AvailableUpgrades {
            stats: 3,
            gold: 0,
            current_shop: None,
        });
    }
}

fn setup_upgrade_ui(mut commands: Commands, upgrades: Res<AvailableUpgrades>, time: Res<Time>) {
    commands.spawn((
        block_root(),
        children![weapons_column(), stats_column(upgrades, time)],
        DespawnOnExit(Screen::Shop),
    ));
}

fn block_root() -> impl Bundle {
    (
        Name::new("Upgrade Menu"),
        Node {
            position_type: PositionType::Absolute,
            display: Display::Grid,
            width: percent(100),
            height: percent(100),
            grid_template_columns: vec![GridTrack::flex(1.0), GridTrack::flex(1.0)],
            ..default()
        },
        // Don't block picking events for other UI roots.
        Pickable::IGNORE,
    )
}

#[derive(Component, Clone)]
enum WeaponButtonMarker {
    Left(u32),
    Right(u32),
    Front,
}

pub fn weapons_column() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Relative,
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            display: Display::Flex,
            ..Default::default()
        },
        Pickable::IGNORE,
        children![
            (label("Weapons")),
            button("Top", weapon_upgrade, WeaponButtonMarker::Front),
            (
                Node {
                    position_type: PositionType::Relative,
                    width: percent(100),
                    height: percent(100),
                    // align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    display: Display::Grid,
                    grid_template_columns: vec![GridTrack::flex(1.0), GridTrack::flex(1.0)],
                    ..Default::default()
                },
                children![
                    (
                        Node {
                            flex_direction: FlexDirection::Column,
                            ..Default::default()
                        },
                        children![
                            button("left_top", weapon_upgrade, WeaponButtonMarker::Left(0)),
                            button("left_middle", weapon_upgrade, WeaponButtonMarker::Left(1)),
                            button("left_bottom", weapon_upgrade, WeaponButtonMarker::Left(2)),
                        ]
                    ),
                    (
                        Node {
                            flex_direction: FlexDirection::Column,
                            ..Default::default()
                        },
                        children![
                            button("right_top", weapon_upgrade, WeaponButtonMarker::Right(0)),
                            button("right_middle", weapon_upgrade, WeaponButtonMarker::Right(1)),
                            button("right_bottom", weapon_upgrade, WeaponButtonMarker::Right(2))
                        ]
                    )
                ]
            ),
        ],
    )
}

fn prng(time: f32) -> u8 {
    ((time * 3_136.234_1).sin().fract() * 10.0) as u8
}

#[derive(Component)]
struct AvailableTextMarker;

fn rng_to_stat(v: u8) -> StatIncreases {
    match v % 4 {
        0 => StatIncreases::ProjectileDamagePercentage,
        1 => StatIncreases::ProjectileSpeedPercentage,
        2 => StatIncreases::ProjectileRatePercentage,
        3 => StatIncreases::ExplosionDamagePercenage,
        _ => unreachable!(),
    }
}

pub fn stats_column(upgrades: Res<AvailableUpgrades>, time: Res<Time>) -> impl Bundle {
    let available = upgrades.stats;
    (
        Node {
            position_type: PositionType::Relative,
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            display: Display::Flex,
            ..Default::default()
        },
        Pickable::IGNORE,
        children![
            (
                label(format!("Available Stats upgrades: {available}")),
                AvailableTextMarker
            ),
            button(
                "Stat1",
                stat_increase,
                rng_to_stat(prng(time.elapsed_secs()))
            ),
            button(
                "Stat2",
                stat_increase,
                rng_to_stat(prng(time.elapsed_secs() + 0.2))
            ),
            button(
                "Stat3",
                stat_increase,
                rng_to_stat(prng(time.elapsed_secs() + 0.3))
            ),
        ],
    )
}

fn update_stat_button_text(
    texts: Query<(&mut Text, &StatIncreases)>,
    players: Query<&PlayerStats>,
) -> Result<(), BevyError> {
    let player = players.single()?;
    let projectile_damage = player.projectile_damage_percentage;
    let projectile_rate = player.projectile_rate_percentage;
    let projectile_speed = player.projectile_speed_percentage;
    let explosion_damage = player.explosion_damage_percentage;
    for (mut text, stat) in texts {
        let stat_text = match stat {
            StatIncreases::ProjectileDamagePercentage => {
                format!("projectile damage ({projectile_damage:.2})")
            }
            StatIncreases::ProjectileSpeedPercentage => {
                format!("projectile speed ({projectile_speed:.2})")
            }
            StatIncreases::ProjectileRatePercentage => {
                format!("projectile rate ({projectile_rate:.2})")
            }
            StatIncreases::ExplosionDamagePercenage => {
                format!("explosion damage ({explosion_damage:.2})")
            }
        };

        text.0 = stat_text;
    }

    Ok(())
}

fn update_upgrade_text(
    mut texts: Query<&mut Text, With<AvailableTextMarker>>,
    upgrades: Res<AvailableUpgrades>,
) {
    for mut text in &mut texts {
        let available = upgrades.stats;
        text.0 = format!("Available Stats upgrades: {available}");
    }
}

fn weapon_upgrade(
    click: On<Pointer<Click>>,
    mut players: Query<&mut WeaponSlots>,
    mut upgrades: ResMut<AvailableUpgrades>,
    buttons: Query<&WeaponButtonMarker>,
) -> Result<(), BevyError> {
    let mut player = players.single_mut()?;
    let selected_weapon_slot = buttons.get(click.entity)?;
    if upgrades.gold >= 2 {
        match selected_weapon_slot {
            WeaponButtonMarker::Left(i) => {
                assert!(*i < 3, "index out of range for weapon access");
                player.left[*i as usize] = Some(WeaponType::default_fire_mage(2));
            }
            WeaponButtonMarker::Right(i) => {
                assert!(*i < 3, "index out of range for weapon access");
                player.right[*i as usize] = Some(WeaponType::default_fire_mage(2));
            }
            WeaponButtonMarker::Front => {
                player.front = Some(WeaponType::default_fire_mage(2));
            }
        }

        upgrades.gold -= 2;
    }

    Ok(())
}

fn update_weapon_button_texts(
    mut texts: Query<(&mut Text, &WeaponButtonMarker)>,
    players: Query<&WeaponSlots>,
    shop: Res<AvailableUpgrades>,
) -> Result<(), BevyError> {
    let player = players.single()?;

    if let Some(available_in_shop) = &shop.current_shop {
        let s = weapon_string(available_in_shop);
        for (mut text, weapon_marker) in &mut texts {
            match weapon_marker {
                WeaponButtonMarker::Left(i) => {
                    assert!(*i < 3, "weapon index out of range");
                    let current = &player.left[*i as usize];
                    if let Some(_current) = current {
                        text.0 = format!("Replace current with {s}")
                    } else {
                        text.0 = format!("Use {s}")
                    }
                }
                WeaponButtonMarker::Right(i) => {
                    assert!(*i < 3, "weapon index out of range");
                    let current = &player.left[*i as usize];
                    if let Some(_current) = current {
                        text.0 = format!("Replace current with {s}")
                    } else {
                        text.0 = format!("Use {s}")
                    }
                }
                WeaponButtonMarker::Front => {
                    if let Some(_current) = &player.front {
                        text.0 = format!("Replace current with {s}")
                    } else {
                        text.0 = format!("Use {s}")
                    }
                }
            }
        }
    } else {
        for (mut text, _) in &mut texts {
            text.0 = "No weapon available".to_string();
        }
    }

    Ok(())
}

fn weapon_string(w: &(WeaponType, u32)) -> String {
    let lvl = w.1;
    match w.0 {
        WeaponType::Canon { .. } => format!("Canon lvl {lvl}"),
        WeaponType::FireMage { .. } => format!("Canon lvl {lvl}"),
        WeaponType::Archer { .. } => format!("Canon lvl {lvl}"),
    }
}

fn stat_increase(
    click: On<Pointer<Click>>,
    mut players: Query<&mut PlayerStats>,
    mut upgrades: ResMut<AvailableUpgrades>,
    source_button_stat: Query<&StatIncreases>,
) -> Result<(), BevyError> {
    if upgrades.stats > 0 {
        let mut player = players.single_mut()?;
        let clicked = source_button_stat.get(click.entity)?;

        match clicked {
            StatIncreases::ProjectileDamagePercentage => player.projectile_damage_percentage += 0.1,
            StatIncreases::ProjectileSpeedPercentage => player.projectile_speed_percentage += 0.1,
            StatIncreases::ProjectileRatePercentage => player.projectile_rate_percentage -= 0.1,
            StatIncreases::ExplosionDamagePercenage => player.explosion_damage_percentage += 0.1,
        }

        upgrades.stats -= 1;
    }

    Ok(())
}
