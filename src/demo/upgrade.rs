use bevy::prelude::*;

use crate::{screens::Screen, theme::widget::label};

pub struct UpgradePlugin;

impl Plugin for UpgradePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Gameplay), setup_upgrade_ui);
    }
}

fn setup_upgrade_ui(mut commands: Commands) {
    commands.spawn((block_root(), children![column(), column()]));
}

fn block_root() -> impl Bundle {
    (
        Name::new("Upgrade Menu"),
        Node {
            position_type: PositionType::Absolute,
            display: Display::Grid,
            width: percent(100),
            height: percent(100),
            grid_template_columns: vec![GridTrack::min_content(), GridTrack::flex(1.0)],
            ..default()
        },
        // Don't block picking events for other UI roots.
        Pickable::IGNORE,
    )
}
pub fn column() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Relative,
            width: percent(45),
            height: percent(100),
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            display: Display::Grid,
            ..Default::default()
        },
        Pickable::IGNORE,
        children![(label("AAAAAAAAAAAAAAAAA"))],
    )
}
