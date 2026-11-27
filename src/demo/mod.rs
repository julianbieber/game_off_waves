//! The game played on the `Screen::Gameplay` screen, and the day that ends in
//! the shop each night.

use std::time::Duration;

use avian2d::prelude::PhysicsLayer;
use bevy::{math::ops::atan2, prelude::*};

use crate::{
    PausableSystems,
    demo::{haven::HavenPlugin, upgrade::UpgradePlugin},
    menus::Menu,
    screens::Screen,
};

pub mod enemy;
pub mod haven;
pub mod level;
mod movement;
pub mod player;
mod terrain;
pub mod upgrade;
pub mod weapons;

#[derive(PhysicsLayer, Default)]
pub enum GameCollisionLayer {
    #[default]
    Terrain,
    Player,
    Enemy,
}

#[derive(Component)]
pub struct Health(i32);

/// The run's day. When `until_night` finishes the shop opens; leaving it starts
/// the next day. `day_counter` is the number of days finished in this run, 0 at
/// the start of a run.
#[derive(Resource)]
pub struct Daytime {
    pub until_night: Timer,
    pub day_counter: u32,
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        movement::plugin,
        player::plugin,
        terrain::TerrainPlugin,
        enemy::EnemyPlugin,
        weapons::WeaponPlugin,
        UpgradePlugin,
        HavenPlugin,
    ));
    app.add_plugins(daytime);
}

fn daytime(app: &mut App) {
    app.add_systems(
        Update,
        (tick_daytime, open_shop_at_night)
            .chain()
            .run_if(in_state(Screen::Gameplay).and_then(in_state(Menu::None)))
            .in_set(PausableSystems),
    );
    app.add_systems(OnEnter(Screen::Gameplay), start_first_day);
    app.add_systems(OnExit(Menu::Shop), start_next_day);
    app.insert_resource(Daytime {
        until_night: Timer::from_seconds(120.0, TimerMode::Once),
        day_counter: 0,
    });
}

fn tick_daytime(time: Res<Time>, mut day: ResMut<Daytime>) {
    day.until_night.tick(time.delta());
}

fn open_shop_at_night(day: Res<Daytime>, mut next_menu: ResMut<NextState<Menu>>) {
    if day.until_night.is_finished() {
        next_menu.set(Menu::Shop);
    }
}

fn start_first_day(mut day: ResMut<Daytime>) {
    day.day_counter = 0;
    start_day(&mut day);
}

fn start_next_day(mut day: ResMut<Daytime>) {
    day.day_counter += 1;
    start_day(&mut day);
}

fn start_day(day: &mut Daytime) {
    day.until_night.reset();
    day.until_night.set_duration(Duration::from_secs(120));
}

pub fn forward_vec(transform: &Transform) -> Vec2 {
    let angle = transform.rotation.to_euler(EulerRot::XYZ).2 + std::f32::consts::FRAC_PI_2;
    Vec2::new(angle.cos(), angle.sin())
}

/// should return the value with lower abs
pub fn angle_between(base: &Transform, point: Vec2) -> f32 {
    let b = (point - base.translation.xy()).normalize();
    let a = forward_vec(base);

    atan2(a.x * b.y - a.y * b.x, a.x * b.x + a.y * b.y)
}

mod test {

    #[allow(unused)]
    use bevy::math::Quat;
    #[allow(unused)]
    use bevy::{math::Vec2, transform::components::Transform};

    #[allow(unused)]
    use crate::demo::angle_between;
    #[allow(unused)]
    use crate::demo::forward_vec;

    #[test]
    fn angle_between_test() {
        let base_pointing_y = Transform::IDENTITY;

        let base_pointing_x =
            Transform::IDENTITY.with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2));

        eq(angle_between(&base_pointing_y, Vec2::Y), 0.0);
        eq(
            angle_between(&base_pointing_y, Vec2::X),
            -std::f32::consts::FRAC_PI_2,
        );
        eq(angle_between(&base_pointing_x, Vec2::X), 0.0);
        eq(
            angle_between(&base_pointing_x, Vec2::Y),
            std::f32::consts::FRAC_PI_2,
        );
    }

    #[allow(unused)]
    fn eq(a: f32, b: f32) {
        dbg!(a, b);
        assert!((a - b).abs() < 0.001);
    }

    #[test]
    fn forward_test() {
        let towards_top = Transform::IDENTITY;
        vec_eq(forward_vec(&towards_top), Vec2::Y);
        let towards_right =
            Transform::IDENTITY.with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2));
        vec_eq(forward_vec(&towards_right), Vec2::X);
        let towards_left =
            Transform::IDENTITY.with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2));
        vec_eq(forward_vec(&towards_left), -Vec2::X);
        let towards_down =
            Transform::IDENTITY.with_rotation(Quat::from_rotation_z(-std::f32::consts::PI));
        vec_eq(forward_vec(&towards_down), -Vec2::Y);
    }

    #[allow(unused)]
    fn vec_eq(a: Vec2, b: Vec2) {
        dbg!(a, b);
        assert!(a.distance(b) < 0.01);
    }
}

#[cfg(test)]
mod daytime_test {
    use std::time::Duration;

    use bevy::{prelude::*, state::app::StatesPlugin};

    use super::{Daytime, daytime};
    use crate::{menus::Menu, screens::Screen};

    fn app_in_gameplay() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<Screen>();
        app.init_state::<Menu>();
        app.add_plugins(daytime);
        app.insert_state(Screen::Gameplay);
        app.update();
        app
    }

    fn finish_day(app: &mut App) {
        let mut day = app.world_mut().resource_mut::<Daytime>();
        let duration = day.until_night.duration();
        day.until_night.tick(duration);
        app.update();
        app.update();
    }

    fn menu(app: &App) -> Menu {
        *app.world().resource::<State<Menu>>().get()
    }

    #[test]
    fn finished_day_opens_the_shop() {
        let mut app = app_in_gameplay();
        assert_eq!(menu(&app), Menu::None);
        finish_day(&mut app);
        assert_eq!(menu(&app), Menu::Shop);
    }

    #[test]
    fn leaving_the_shop_starts_the_next_day() {
        let mut app = app_in_gameplay();
        finish_day(&mut app);
        app.world_mut()
            .resource_mut::<NextState<Menu>>()
            .set(Menu::None);
        app.update();
        app.update();
        let day = app.world().resource::<Daytime>();
        assert_eq!(menu(&app), Menu::None);
        assert_eq!(day.day_counter, 1);
        assert!(!day.until_night.is_finished());
        assert!(day.until_night.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn new_run_starts_at_day_zero() {
        let mut app = app_in_gameplay();
        app.world_mut().resource_mut::<Daytime>().day_counter = 4;
        app.world_mut()
            .resource_mut::<NextState<Screen>>()
            .set(Screen::Title);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<Screen>>()
            .set(Screen::Gameplay);
        app.update();
        assert_eq!(app.world().resource::<Daytime>().day_counter, 0);
    }
}
