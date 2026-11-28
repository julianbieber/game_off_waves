//! The pause menu.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::{NoMarker, menus::Menu, screens::Screen, theme::widget};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Pause), spawn_pause_menu);
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Pause).and_then(input_just_pressed(KeyCode::Escape))),
    );
}

fn spawn_pause_menu(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        @widget::ui_root("Pause Menu")
        GlobalZIndex(2)
        ~{DespawnOnExit(Menu::Pause)}
        Children [
            @widget::header("Game paused")
            --
            @widget::button("Continue", close_menu, NoMarker)
            --
            @widget::button("Settings", open_settings_menu, NoMarker)
            --
            @widget::button("Quit to title", quit_to_title, NoMarker)
        ]
    });
}

fn open_settings_menu(_: On<PointerClick>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

fn close_menu(_: On<PointerClick>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}

fn quit_to_title(_: On<PointerClick>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

fn go_back(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
