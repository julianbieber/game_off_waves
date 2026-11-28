//! The shop that opens at night, where the player spends kill gold and stat
//! points on the ship.

use bevy::prelude::*;

use crate::{
    NoMarker,
    demo::{
        Daytime,
        player::{PlayerStats, StatIncreases},
        weapons::{Hold, Slot, Weapon, WeaponSlots, WeaponType},
    },
    menus::Menu,
    theme::{
        palette::{BUTTON_BACKGROUND, BUTTON_TEXT},
        widget::{button, label},
    },
};

pub struct UpgradePlugin;

#[derive(Resource)]
pub struct AvailableUpgrades {
    pub stats: u32,
    pub gold: u32,
    pub current_shop: Option<Weapon>,
}

impl Plugin for UpgradePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Menu::Shop), (roll_offer, setup_upgrade_ui));
        app.add_systems(
            Update,
            (
                update_upgrade_text,
                update_stat_button_text,
                update_offer_text,
                update_slot_texts,
                refresh_hold_list,
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

#[derive(Component, Clone, Copy)]
struct SlotCell(Slot);

#[derive(Component, Clone, Copy)]
struct SlotText(Slot);

#[derive(Component, Clone, Copy)]
struct HoldEntry(usize);

#[derive(Component, Clone, Default)]
struct HoldList;

#[derive(Component, Clone, Default)]
struct OfferButton;

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
            row_gap: px(8),
        }
        Pickable::IGNORE
        Children [
            @button("", buy_offer, OfferButton)
            --
            @label("Weapons")
            --
            @slot_cell(Slot::Front)
            --
            Node {
                display: Display::Grid,
                grid_template_columns: vec![GridTrack::flex(1.0), GridTrack::flex(1.0)],
                column_gap: px(8),
            }
            Pickable::IGNORE
            Children [
                Node { flex_direction: FlexDirection::Column, row_gap: px(8) }
                Pickable::IGNORE
                Children [
                    @slot_cell(Slot::Left(0))
                    --
                    @slot_cell(Slot::Left(1))
                    --
                    @slot_cell(Slot::Left(2))
                ]
                --
                Node { flex_direction: FlexDirection::Column, row_gap: px(8) }
                Pickable::IGNORE
                Children [
                    @slot_cell(Slot::Right(0))
                    --
                    @slot_cell(Slot::Right(1))
                    --
                    @slot_cell(Slot::Right(2))
                ]
            ]
            --
            @label("Hold")
            --
            HoldList
            Node {
                width: percent(90),
                min_height: px(64),
                flex_wrap: FlexWrap::Wrap,
                column_gap: px(8),
                row_gap: px(8),
                padding: UiRect::all(px(8)),
            }
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3))
            on(drop_into_hold)
        ]
    }
}

fn slot_cell(slot: Slot) -> impl Scene {
    bsn! {
        ~{SlotCell(slot)}
        Node {
            width: px(260),
            height: px(56),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        }
        BackgroundColor(BUTTON_BACKGROUND)
        @draggable()
        on(drop_onto_slot)
        Children [
            ~{SlotText(slot)}
            Text
            TextFont { font_size: FontSize::Px(22.0) }
            TextColor(BUTTON_TEXT)
            ~{Pickable::IGNORE}
        ]
    }
}

fn hold_entry(index: usize, weapon: &Weapon) -> impl Scene {
    let text = Text(weapon_label(weapon));
    bsn! {
        ~{HoldEntry(index)}
        Node {
            padding: UiRect::axes(px(12), px(8)),
            align_items: AlignItems::Center,
        }
        BackgroundColor(BUTTON_BACKGROUND)
        @draggable()
        Children [
            ~{text}
            TextFont { font_size: FontSize::Px(22.0) }
            TextColor(BUTTON_TEXT)
            ~{Pickable::IGNORE}
        ]
    }
}

fn draggable() -> impl Scene {
    bsn! {
        ~{Pickable { should_block_lower: false, is_hoverable: true }}
        UiTransform
        GlobalZIndex
        on(lift)
        on(follow_pointer)
        on(set_down)
    }
}

fn lift(start: On<PointerDragStart>, mut layers: Query<&mut GlobalZIndex>) {
    if let Ok(mut layer) = layers.get_mut(start.event_target()) {
        layer.0 = 1;
    }
}

fn follow_pointer(drag: On<PointerDrag>, mut transforms: Query<&mut UiTransform>) {
    if let Ok(mut transform) = transforms.get_mut(drag.event_target()) {
        transform.translation = Val2::px(drag.distance.x, drag.distance.y);
    }
}

fn set_down(end: On<PointerDragEnd>, mut dragged: Query<(&mut UiTransform, &mut GlobalZIndex)>) {
    if let Ok((mut transform, mut layer)) = dragged.get_mut(end.event_target()) {
        transform.translation = Val2::ZERO;
        layer.0 = 0;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Source {
    Slot(Slot),
    Hold(usize),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Target {
    Slot(Slot),
    Hold,
}

fn move_weapon(slots: &mut WeaponSlots, hold: &mut Hold, from: Source, to: Target) {
    let weapon = match (from, to) {
        (Source::Slot(a), Target::Slot(b)) if a == b => return,
        (Source::Hold(_), Target::Hold) => return,
        (Source::Slot(slot), _) => slots.take(slot),
        (Source::Hold(i), _) if i < hold.0.len() => Some(hold.0.remove(i)),
        (Source::Hold(_), _) => None,
    };
    let Some(weapon) = weapon else { return };
    match to {
        Target::Slot(slot) => hold.0.extend(slots.put(slot, weapon)),
        Target::Hold => hold.0.push(weapon),
    }
}

fn drag_source(
    dropped: Entity,
    cells: &Query<&SlotCell>,
    entries: &Query<&HoldEntry>,
) -> Option<Source> {
    if let Ok(cell) = cells.get(dropped) {
        return Some(Source::Slot(cell.0));
    }
    entries.get(dropped).ok().map(|entry| Source::Hold(entry.0))
}

fn drop_onto_slot(
    drop: On<PointerDragDrop>,
    cells: Query<&SlotCell>,
    entries: Query<&HoldEntry>,
    mut ship: Single<(&mut WeaponSlots, &mut Hold)>,
) {
    let (Some(from), Ok(target)) = (
        drag_source(drop.dropped, &cells, &entries),
        cells.get(drop.event_target()),
    ) else {
        return;
    };
    let (slots, hold) = &mut *ship;
    move_weapon(slots, hold, from, Target::Slot(target.0));
}

fn drop_into_hold(
    drop: On<PointerDragDrop>,
    cells: Query<&SlotCell>,
    entries: Query<&HoldEntry>,
    mut ship: Single<(&mut WeaponSlots, &mut Hold)>,
) {
    let Some(from) = drag_source(drop.dropped, &cells, &entries) else {
        return;
    };
    let (slots, hold) = &mut *ship;
    move_weapon(slots, hold, from, Target::Hold);
}

fn refresh_hold_list(
    hold: Single<Ref<Hold>>,
    list: Single<(Entity, Ref<HoldList>)>,
    entries: Query<Entity, With<HoldEntry>>,
    mut commands: Commands,
) {
    let (list, marker) = *list;
    if !hold.is_changed() && !marker.is_added() {
        return;
    }
    for entry in &entries {
        commands.entity(entry).despawn();
    }
    for (index, weapon) in hold.0.iter().enumerate() {
        commands
            .spawn_scene(hold_entry(index, weapon))
            .insert(ChildOf(list));
    }
}

fn update_slot_texts(mut texts: Query<(&mut Text, &SlotText)>, slots: Single<&WeaponSlots>) {
    for (mut text, slot) in &mut texts {
        let held = slots
            .get(slot.0)
            .map_or_else(|| "empty".to_string(), weapon_label);
        text.0 = format!("{}: {held}", slot_name(slot.0));
    }
}

fn slot_name(slot: Slot) -> String {
    match slot {
        Slot::Front => "Front".to_string(),
        Slot::Left(i) => format!("Left {}", i + 1),
        Slot::Right(i) => format!("Right {}", i + 1),
    }
}

fn weapon_label(weapon: &Weapon) -> String {
    format!("{} lvl {}", weapon.kind.name(), weapon.level)
}

fn weapon_price(level: u32) -> u32 {
    (2.0 * 1.1_f32.powi(level as i32)).round() as u32
}

fn roll_offer(mut upgrades: ResMut<AvailableUpgrades>, day: Res<Daytime>, time: Res<Time>) {
    let level = day.day_counter + 1;
    upgrades.current_shop = Some(match prng(time.elapsed_secs() + 0.7) % 3 {
        0 => WeaponType::default_cannon(level),
        1 => WeaponType::default_fire_mage(level),
        _ => WeaponType::default_archer(level),
    });
}

fn buy_offer(
    _: On<PointerClick>,
    mut upgrades: ResMut<AvailableUpgrades>,
    mut hold: Single<&mut Hold>,
) {
    let Some(price) = upgrades
        .current_shop
        .as_ref()
        .map(|w| weapon_price(w.level))
    else {
        return;
    };
    if upgrades.gold < price {
        return;
    }
    upgrades.gold -= price;
    hold.0.extend(upgrades.current_shop.take());
}

fn update_offer_text(
    mut texts: Query<&mut Text, With<OfferButton>>,
    upgrades: Res<AvailableUpgrades>,
) {
    let gold = upgrades.gold;
    let offer = match &upgrades.current_shop {
        Some(weapon) => format!(
            "Buy {} for {} gold ({gold})",
            weapon_label(weapon),
            weapon_price(weapon.level)
        ),
        None => format!("Sold out ({gold} gold)"),
    };
    for mut text in &mut texts {
        text.0.clone_from(&offer);
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
        app_in_shop_on_day(0)
    }

    fn app_in_shop_on_day(day_counter: u32) -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            StatesPlugin,
            AssetPlugin::default(),
            ScenePlugin,
        ));
        app.init_state::<Menu>();
        app.insert_resource(Daytime {
            until_night: Timer::from_seconds(120.0, TimerMode::Once),
            day_counter,
        });
        app.add_plugins(UpgradePlugin);
        app.world_mut().spawn((
            PlayerStats::default(),
            WeaponSlots {
                left: [None, None, None],
                right: [None, None, None],
                front: None,
            },
            Hold::default(),
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

    fn ship(app: &mut App) -> (Vec<Option<u32>>, Vec<u32>) {
        let mut ships = app.world_mut().query::<(&WeaponSlots, &Hold)>();
        let (slots, hold) = ships.single(app.world()).unwrap();
        (
            Slot::ALL
                .iter()
                .map(|s| slots.get(*s).map(|w| w.level))
                .collect(),
            hold.0.iter().map(|w| w.level).collect(),
        )
    }

    fn drop_on(app: &mut App, dropped: Entity, target: Entity) {
        let location = Location {
            target: NormalizedRenderTarget::None {
                width: 1,
                height: 1,
            },
            position: Vec2::ZERO,
        };
        app.world_mut().trigger(PointerDragDrop {
            entity: target,
            pointer: Pointer::new(PointerId::Mouse, location),
            button: PointerButton::Primary,
            dropped,
            hit: HitData::new(target, 0.0, None, None),
        });
        app.update();
    }

    fn cell(app: &mut App, slot: Slot) -> Entity {
        let mut cells = app.world_mut().query::<(Entity, &SlotCell)>();
        cells
            .iter(app.world())
            .find(|(_, c)| c.0 == slot)
            .map(|(e, _)| e)
            .expect("no such slot in the shop")
    }

    fn hold_entry_entity(app: &mut App, index: usize) -> Entity {
        let mut entries = app.world_mut().query::<(Entity, &HoldEntry)>();
        entries
            .iter(app.world())
            .find(|(_, h)| h.0 == index)
            .map(|(e, _)| e)
            .expect("no such hold entry in the shop")
    }

    #[test]
    fn the_offer_is_at_the_next_nights_level() {
        let app = app_in_shop_on_day(4);
        let offer = app.world().resource::<AvailableUpgrades>();
        assert_eq!(offer.current_shop.as_ref().unwrap().level, 5);
    }

    #[test]
    fn the_price_grows_slowly_and_exponentially() {
        let prices: Vec<u32> = [1, 5, 10, 20, 30].map(weapon_price).into();
        assert_eq!(prices, vec![2, 3, 5, 13, 35]);
    }

    #[test]
    fn buying_moves_the_offer_into_the_hold_and_sells_out() {
        let mut app = app_in_shop();
        app.world_mut().resource_mut::<AvailableUpgrades>().gold = 5;
        let buy = button_with::<OfferButton>(&mut app, |_| true);
        click(&mut app, buy);
        let shop = app.world().resource::<AvailableUpgrades>();
        assert!(shop.current_shop.is_none());
        assert_eq!(shop.gold, 5 - weapon_price(1));
        assert_eq!(ship(&mut app).1, vec![1]);
        click(&mut app, buy);
        assert_eq!(
            app.world().resource::<AvailableUpgrades>().gold,
            5 - weapon_price(1)
        );
    }

    #[test]
    fn buying_without_enough_gold_changes_nothing() {
        let mut app = app_in_shop();
        let buy = button_with::<OfferButton>(&mut app, |_| true);
        click(&mut app, buy);
        assert!(
            app.world()
                .resource::<AvailableUpgrades>()
                .current_shop
                .is_some()
        );
        assert!(ship(&mut app).1.is_empty());
    }

    #[test]
    fn dropping_a_hold_weapon_onto_a_full_slot_sends_the_old_one_to_the_hold() {
        let mut app = app_in_shop();
        give(&mut app, Slot::Left(1), 7);
        stow(&mut app, 3);
        let entry = hold_entry_entity(&mut app, 0);
        let target = cell(&mut app, Slot::Left(1));
        drop_on(&mut app, entry, target);
        let (slots, hold) = ship(&mut app);
        assert_eq!(slots[2], Some(3));
        assert_eq!(hold, vec![7]);
    }

    #[test]
    fn dropping_a_slot_weapon_onto_another_slot_moves_it_and_holds_the_displaced() {
        let mut app = app_in_shop();
        give(&mut app, Slot::Front, 1);
        give(&mut app, Slot::Right(2), 2);
        let from = cell(&mut app, Slot::Front);
        let to = cell(&mut app, Slot::Right(2));
        drop_on(&mut app, from, to);
        let (slots, hold) = ship(&mut app);
        assert_eq!(slots[0], None);
        assert_eq!(slots[6], Some(1));
        assert_eq!(hold, vec![2]);
    }

    #[test]
    fn dropping_a_slot_weapon_into_the_hold_empties_the_slot() {
        let mut app = app_in_shop();
        give(&mut app, Slot::Right(0), 4);
        let from = cell(&mut app, Slot::Right(0));
        let hold_list = {
            let mut lists = app.world_mut().query_filtered::<Entity, With<HoldList>>();
            lists.single(app.world()).unwrap()
        };
        drop_on(&mut app, from, hold_list);
        let (slots, hold) = ship(&mut app);
        assert_eq!(slots[4], None);
        assert_eq!(hold, vec![4]);
    }

    #[test]
    fn the_hold_list_shows_one_entry_per_held_weapon() {
        let mut app = app_in_shop();
        stow(&mut app, 1);
        stow(&mut app, 2);
        let mut entries = app.world_mut().query::<&HoldEntry>();
        assert_eq!(entries.iter(app.world()).count(), 2);
    }

    fn give(app: &mut App, slot: Slot, level: u32) {
        let mut slots = app.world_mut().query::<&mut WeaponSlots>();
        slots
            .single_mut(app.world_mut())
            .unwrap()
            .put(slot, WeaponType::default_cannon(level));
        app.update();
    }

    fn stow(app: &mut App, level: u32) {
        let mut holds = app.world_mut().query::<&mut Hold>();
        holds
            .single_mut(app.world_mut())
            .unwrap()
            .0
            .push(WeaponType::default_archer(level));
        app.update();
    }

    #[test]
    fn next_day_button_leaves_the_shop() {
        let mut app = app_in_shop();
        let next_day = button_with::<NoMarker>(&mut app, |_| true);
        click(&mut app, next_day);
        assert_eq!(*app.world().resource::<State<Menu>>().get(), Menu::None);
    }

    #[test]
    fn every_shop_button_and_slot_shows_its_label() {
        let mut app = app_in_shop();
        let mut texts = app
            .world_mut()
            .query_filtered::<&Text, Or<(With<OfferButton>, With<SlotText>, With<StatIncreases>)>>(
            );
        let labels: Vec<String> = texts.iter(app.world()).map(|t| t.0.clone()).collect();
        assert_eq!(labels.len(), 11);
        assert!(labels.iter().all(|l| !l.is_empty()));
    }
}
