use bevy::prelude::*;

use crate::{
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
        app.add_systems(Update, update_upgrade_text);
        app.insert_resource(AvailableUpgrades { stats: 3, _gold: 0 });
    }
}

fn setup_upgrade_ui(mut commands: Commands, upgrades: Res<AvailableUpgrades>) {
    commands.spawn((
        block_root(),
        children![weapons_column(), stats_column(upgrades)],
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
            button("Top", tmp_click),
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
                            button("left_top", tmp_click),
                            button("left_middle", tmp_click),
                            button("left_bottom", tmp_click),
                        ]
                    ),
                    (
                        Node {
                            flex_direction: FlexDirection::Column,
                            ..Default::default()
                        },
                        children![
                            button("right_top", tmp_click),
                            button("right_middle", tmp_click),
                            button("right_bottom", tmp_click)
                        ]
                    )
                ]
            ),
        ],
    )
}

fn _prng(time: &Time) -> u8 {
    ((time.elapsed_secs() * 312936.234114).sin().fract() * 10.0) as u8
}

#[derive(Component)]
struct AvalableTextMarker;

pub fn stats_column(upgrades: Res<AvailableUpgrades>) -> impl Bundle {
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
            (
                button("Stat1", stat_increase),
                StatIncreases::ProjectileDamagePercentage
            ),
            (
                button("Stat2", stat_increase),
                StatIncreases::ProjectileSpeedPercentage
            ),
            (
                button("Stat3", stat_increase),
                StatIncreases::ProjectileRatePercentage
            ),
        ],
    )
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
    _: On<Pointer<Click>>,
    mut players: Query<&mut PlayerStats>,
    mut upgrades: ResMut<AvailableUpgrades>,
) -> Result<(), BevyError> {
    if upgrades.stats > 0 {
        let mut player = players.single_mut()?;

        player.explosion_damage_percentage *= 1.1;
        upgrades.stats -= 1;
    }

    Ok(())
}
