//! The main menu (seen on the title screen).

use bevy::prelude::*;

use crate::{
    NoMarker, asset_tracking::ResourceHandles, menus::Menu, screens::Screen, theme::widget,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Main), spawn_main_menu);
}

fn spawn_main_menu(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        @widget::ui_root("Main Menu")
        GlobalZIndex(2)
        ~{DespawnOnExit(Menu::Main)}
        Children [ {main_menu_buttons()} ]
    });
}

#[cfg(not(target_family = "wasm"))]
fn main_menu_buttons() -> impl SceneList {
    bsn_list! {
        @widget::button("Play", enter_loading_or_gameplay_screen, NoMarker)
        --
        @widget::button("Settings", open_settings_menu, NoMarker)
        --
        @widget::button("Credits", open_credits_menu, NoMarker)
        --
        @widget::button("Exit", exit_app, NoMarker)
    }
}

#[cfg(target_family = "wasm")]
fn main_menu_buttons() -> impl SceneList {
    bsn_list! {
        @widget::button("Play", enter_loading_or_gameplay_screen, NoMarker)
        --
        @widget::button("Settings", open_settings_menu, NoMarker)
        --
        @widget::button("Credits", open_credits_menu, NoMarker)
    }
}

fn enter_loading_or_gameplay_screen(
    _: On<PointerClick>,
    resource_handles: Res<ResourceHandles>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    if resource_handles.is_all_done() {
        next_screen.set(Screen::Gameplay);
    } else {
        next_screen.set(Screen::Loading);
    }
}

fn open_settings_menu(_: On<PointerClick>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

fn open_credits_menu(_: On<PointerClick>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Credits);
}

#[cfg(not(target_family = "wasm"))]
fn exit_app(_: On<PointerClick>, mut app_exit: MessageWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}
