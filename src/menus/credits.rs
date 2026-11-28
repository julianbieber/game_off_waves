//! The credits menu.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::{NoMarker, asset_tracking::LoadResource, audio::music, menus::Menu, theme::prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Credits), spawn_credits_menu);
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Credits).and_then(input_just_pressed(KeyCode::Escape))),
    );

    app.load_resource::<CreditsAssets>();
    app.add_systems(OnEnter(Menu::Credits), start_credits_music);
}

fn spawn_credits_menu(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        @widget::ui_root("Credits Menu")
        GlobalZIndex(2)
        ~{DespawnOnExit(Menu::Credits)}
        Children [
            @widget::header("Created by")
            --
            @created_by()
            --
            @widget::header("Assets")
            --
            @assets()
            --
            @widget::button("Back", go_back_on_click, NoMarker)
        ]
    });
}

fn created_by() -> impl Scene {
    grid(vec![
        ["Joe Shmoe", "Implemented alligator wrestling AI"],
        ["Jane Doe", "Made the music for the alien invasion"],
    ])
}

fn assets() -> impl Scene {
    grid(vec![
        ["Ducky sprite", "CC0 by Caz Creates Games"],
        ["Button SFX", "CC0 by Jaszunio15"],
        ["Music", "CC BY 3.0 by Kevin MacLeod"],
        [
            "Bevy logo",
            "All rights reserved by the Bevy Foundation, permission granted for splash screen use when unmodified",
        ],
    ])
}

fn grid(content: Vec<[&'static str; 2]>) -> impl Scene {
    let cells = content
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(i, text)| grid_cell(text, i.is_multiple_of(2)))
        .collect::<Vec<_>>();
    bsn! {
        #Grid
        Node {
            display: Display::Grid,
            row_gap: px(10),
            column_gap: px(30),
            grid_template_columns: RepeatedGridTrack::px::<Vec<RepeatedGridTrack>>(2, 400.0),
        }
        Children [ {cells} ]
    }
}

fn grid_cell(text: &'static str, first_column: bool) -> impl Scene {
    let justify_self = if first_column {
        JustifySelf::End
    } else {
        JustifySelf::Start
    };
    bsn! {
        @widget::label(text)
        Node { justify_self: justify_self }
    }
}

fn go_back_on_click(_: On<PointerClick>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

fn go_back(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
struct CreditsAssets {
    #[dependency]
    music: Handle<AudioSource>,
}

impl FromWorld for CreditsAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/Monkeys Spinning Monkeys.ogg"),
        }
    }
}

fn start_credits_music(mut commands: Commands, credits_music: Res<CreditsAssets>) {
    commands.spawn((
        Name::new("Credits Music"),
        DespawnOnExit(Menu::Credits),
        music(credits_music.music.clone()),
    ));
}
