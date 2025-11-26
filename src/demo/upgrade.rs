use bevy::prelude::*;

use crate::{
    NoMarker,
    demo::player::{PlayerStats, StatIncreases},
    screens::Screen,
    theme::widget::{button, label},
};

pub struct UpgradePlugin;

#[derive(Resource)]
pub struct AvailableUpgrades {
    pub stats: u32,
    pub _gold: u32,
}

impl Plugin for UpgradePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Gameplay), setup_upgrade_ui);
        app.add_systems(Update, (update_upgrade_text, update_stat_button_text));
        app.insert_resource(AvailableUpgrades { stats: 3, _gold: 0 });
    }
}

fn setup_upgrade_ui(mut commands: Commands, upgrades: Res<AvailableUpgrades>, time: Res<Time>) {
    commands.spawn((
        block_root(),
        children![weapons_column(), stats_column(upgrades, time)],
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
            button("Top", tmp_click, NoMarker),
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
                            button("left_top", tmp_click, NoMarker),
                            button("left_middle", tmp_click, NoMarker),
                            button("left_bottom", tmp_click, NoMarker),
                        ]
                    ),
                    (
                        Node {
                            flex_direction: FlexDirection::Column,
                            ..Default::default()
                        },
                        children![
                            button("right_top", tmp_click, NoMarker),
                            button("right_middle", tmp_click, NoMarker),
                            button("right_bottom", tmp_click, NoMarker)
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
struct AvalableTextMarker;

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
                AvalableTextMarker
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
                format!("projectile damage ({projectile_damage})")
            }
            StatIncreases::ProjectileSpeedPercentage => {
                format!("projectile speed ({projectile_speed})")
            }
            StatIncreases::ProjectileRatePercentage => {
                format!("projectile damage ({projectile_rate})")
            }
            StatIncreases::ExplosionDamagePercenage => {
                format!("projectile damage ({explosion_damage})")
            }
        };

        text.0 = stat_text;
    }

    Ok(())
}

fn update_upgrade_text(
    mut texts: Query<&mut Text, With<AvalableTextMarker>>,
    upgrades: Res<AvailableUpgrades>,
) {
    for mut text in &mut texts {
        let available = upgrades.stats;
        text.0 = format!("Available Stats upgrades: {available}");
    }
}

fn tmp_click(_: On<Pointer<Click>>) {
    warn!("click");
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
            StatIncreases::ProjectileRatePercentage => player.projectile_rate_percentage += 0.1,
            StatIncreases::ExplosionDamagePercenage => player.explosion_damage_percentage += 0.1,
        }

        upgrades.stats -= 1;
    }

    Ok(())
}
