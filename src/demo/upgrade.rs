//! The shop that opens at night: kill gold and stat points are spent here, and
//! "Next day" leaves it.

use bevy::prelude::*;

use crate::{
    NoMarker,
    demo::{
        player::{PlayerStats, StatIncreases},
        weapons::{WeaponSlots, WeaponType},
    },
    menus::Menu,
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
        app.add_systems(OnEnter(Menu::Shop), setup_upgrade_ui);
        app.add_systems(
            Update,
            (
                update_upgrade_text,
                update_stat_button_text,
                update_weapon_button_texts,
            )
                .run_if(in_state(Menu::Shop)),
        );
        app.insert_resource(AvailableUpgrades {
            stats: 3,
            gold: 0,
            current_shop: None,
        });
    }
}

fn setup_upgrade_ui(mut commands: Commands, upgrades: Res<AvailableUpgrades>, time: Res<Time>) {
    let available = upgrades.stats;
    let elapsed = time.elapsed_secs();
    commands.spawn_scene(bsn! {
        @block_root()
        ~{DespawnOnExit(Menu::Shop)}
        Children [
            @weapons_column()
            --
            @stats_column(available, elapsed)
        ]
    });
}

fn block_root() -> impl Scene {
    bsn! {
        ~{Name::new("Upgrade Menu")}
        Node {
            position_type: PositionType::Absolute,
            display: Display::Grid,
            width: percent(100),
            height: percent(100),
            grid_template_columns: vec![GridTrack::flex(1.0), GridTrack::flex(1.0)],
        }
        Pickable::IGNORE
    }
}

#[derive(Component, Clone)]
enum WeaponButtonMarker {
    Left(u32),
    Right(u32),
    Front,
}

fn weapons_column() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Relative,
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            display: Display::Flex,
        }
        Pickable::IGNORE
        Children [
            @label("Weapons")
            --
            @button("Top", weapon_upgrade, WeaponButtonMarker::Front)
            --
            Node {
                position_type: PositionType::Relative,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                display: Display::Grid,
                grid_template_columns: vec![GridTrack::flex(1.0), GridTrack::flex(1.0)],
            }
            Children [
                Node { flex_direction: FlexDirection::Column }
                Children [
                    @button("left_top", weapon_upgrade, WeaponButtonMarker::Left(0))
                    --
                    @button("left_middle", weapon_upgrade, WeaponButtonMarker::Left(1))
                    --
                    @button("left_bottom", weapon_upgrade, WeaponButtonMarker::Left(2))
                ]
                --
                Node { flex_direction: FlexDirection::Column }
                Children [
                    @button("right_top", weapon_upgrade, WeaponButtonMarker::Right(0))
                    --
                    @button("right_middle", weapon_upgrade, WeaponButtonMarker::Right(1))
                    --
                    @button("right_bottom", weapon_upgrade, WeaponButtonMarker::Right(2))
                ]
            ]
        ]
    }
}

fn leave_shop(_: On<PointerClick>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}

fn prng(time: f32) -> u8 {
    ((time * 3_136.234_1).sin().fract() * 10.0) as u8
}

#[derive(Component, Clone, Default)]
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

fn stats_column(available: u32, elapsed: f32) -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Relative,
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            display: Display::Flex,
        }
        Pickable::IGNORE
        Children [
            @label(format!("Available Stats upgrades: {available}"))
            AvailableTextMarker
            --
            @button("Stat1", stat_increase, rng_to_stat(prng(elapsed)))
            --
            @button("Stat2", stat_increase, rng_to_stat(prng(elapsed + 0.2)))
            --
            @button("Stat3", stat_increase, rng_to_stat(prng(elapsed + 0.3)))
            --
            @button("Next day", leave_shop, NoMarker)
        ]
    }
}

fn update_stat_button_text(
    texts: Query<(&mut Text, &StatIncreases)>,
    player: Single<&PlayerStats>,
) {
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
    click: On<PointerClick>,
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
    player: Single<&WeaponSlots>,
    shop: Res<AvailableUpgrades>,
) {
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
    click: On<PointerClick>,
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::{
        asset::AssetPlugin,
        camera::NormalizedRenderTarget,
        picking::{
            backend::HitData,
            events::Pointer,
            pointer::{Location, PointerButton, PointerId},
        },
        scene::ScenePlugin,
        state::app::StatesPlugin,
        ui_widgets::Button,
    };

    use super::*;

    fn app_in_shop() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            StatesPlugin,
            AssetPlugin::default(),
            ScenePlugin,
        ));
        app.init_state::<Menu>();
        app.add_plugins(UpgradePlugin);
        app.world_mut().spawn((
            PlayerStats::default(),
            WeaponSlots {
                left: [None, None, None],
                right: [None, None, None],
                front: None,
            },
        ));
        app.insert_state(Menu::Shop);
        app.update();
        app
    }

    fn click(app: &mut App, entity: Entity) {
        let location = Location {
            target: NormalizedRenderTarget::None {
                width: 1,
                height: 1,
            },
            position: Vec2::ZERO,
        };
        app.world_mut().trigger(PointerClick {
            entity,
            pointer: Pointer::new(PointerId::Mouse, location),
            button: PointerButton::Primary,
            hit: HitData::new(entity, 0.0, None, None),
            duration: Duration::ZERO,
            count: 1,
        });
        app.update();
    }

    fn button_with<C: Component>(app: &mut App, matches: impl Fn(&C) -> bool) -> Entity {
        let mut buttons = app
            .world_mut()
            .query_filtered::<(Entity, &C), With<Button>>();
        buttons
            .iter(app.world())
            .find(|(_, c)| matches(c))
            .map(|(e, _)| e)
            .expect("no such button in the shop")
    }

    #[test]
    fn stat_button_spends_a_stat_point() {
        let mut app = app_in_shop();
        let stat = button_with::<StatIncreases>(&mut app, |_| true);
        click(&mut app, stat);
        assert_eq!(app.world().resource::<AvailableUpgrades>().stats, 2);
    }

    #[test]
    fn weapon_slot_button_places_a_weapon_for_two_gold() {
        let mut app = app_in_shop();
        app.world_mut().resource_mut::<AvailableUpgrades>().gold = 2;
        let slot = button_with::<WeaponButtonMarker>(&mut app, |m| {
            matches!(m, WeaponButtonMarker::Left(1))
        });
        click(&mut app, slot);
        let mut slots = app.world_mut().query::<&WeaponSlots>();
        assert!(slots.single(app.world()).unwrap().left[1].is_some());
        assert_eq!(app.world().resource::<AvailableUpgrades>().gold, 0);
    }

    #[test]
    fn next_day_button_leaves_the_shop() {
        let mut app = app_in_shop();
        let next_day = button_with::<NoMarker>(&mut app, |_| true);
        click(&mut app, next_day);
        assert_eq!(*app.world().resource::<State<Menu>>().get(), Menu::None);
    }

    #[test]
    fn every_shop_button_shows_its_label() {
        let mut app = app_in_shop();
        let mut texts = app
            .world_mut()
            .query_filtered::<&Text, Or<(With<WeaponButtonMarker>, With<StatIncreases>)>>();
        let labels: Vec<String> = texts.iter(app.world()).map(|t| t.0.clone()).collect();
        assert_eq!(labels.len(), 10);
        assert!(labels.iter().all(|l| !l.is_empty()));
    }
}
