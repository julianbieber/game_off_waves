//! Scenes for the common widgets every screen and menu is built from.

use std::borrow::Cow;

use bevy::{
    ecs::{event::EventPattern, system::IntoObserverSystem},
    prelude::*,
    ui_widgets::Button,
};

use crate::theme::{interaction::InteractionPalette, palette::*};

/// A root UI node that fills the window and centers its content.
pub fn ui_root(name: impl Into<Cow<'static, str>>) -> impl Scene {
    let name = Name::new(name);
    bsn! {
        ~{name}
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(20),
        }
        ~{Pickable::IGNORE}
    }
}

/// A simple header label. Bigger than [`label`].
pub fn header(text: impl Into<String>) -> impl Scene {
    let text = Text(text.into());
    bsn! {
        #Header
        ~{text}
        TextFont { font_size: FontSize::Px(40.0) }
        TextColor(HEADER_TEXT)
    }
}

/// A simple text label.
pub fn label(text: impl Into<String>) -> impl Scene {
    let text = Text(text.into());
    bsn! {
        #Label
        ~{text}
        TextFont { font_size: FontSize::Px(24.0) }
        TextColor(LABEL_TEXT)
    }
}

/// A large rounded button with text and an action defined as an [`Observer`].
/// `marker` goes on both the button and its text.
pub fn button<E, M, I>(
    text: impl Into<String>,
    action: I,
    marker: impl Component + Clone + Unpin,
) -> impl Scene
where
    E: EventPattern<Event: EntityEvent>,
    I: IntoObserverSystem<E, M> + Clone + Send + Sync,
    M: 'static,
{
    button_base(
        text,
        action,
        Node {
            width: px(380),
            height: px(80),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::MAX,
            ..default()
        },
        marker,
    )
}

/// A small square button with text and an action defined as an [`Observer`].
/// `marker` goes on both the button and its text.
pub fn button_small<E, M, I>(
    text: impl Into<String>,
    action: I,
    marker: impl Component + Clone + Unpin,
) -> impl Scene
where
    E: EventPattern<Event: EntityEvent>,
    I: IntoObserverSystem<E, M> + Clone + Send + Sync,
    M: 'static,
{
    button_base(
        text,
        action,
        Node {
            width: px(30),
            height: px(30),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        marker,
    )
}

fn button_base<E, M, I>(
    text: impl Into<String>,
    action: I,
    layout: Node,
    marker: impl Component + Clone + Unpin,
) -> impl Scene
where
    E: EventPattern<Event: EntityEvent>,
    I: IntoObserverSystem<E, M> + Clone + Send + Sync,
    M: 'static,
{
    let text = Text(text.into());
    let text_marker = marker.clone();
    bsn! {
        #Button
        Node
        Children [
            ~{Name::new("Button Inner")}
            Button
            BackgroundColor(BUTTON_BACKGROUND)
            InteractionPalette {
                none: BUTTON_BACKGROUND,
                hovered: BUTTON_HOVERED_BACKGROUND,
                pressed: BUTTON_PRESSED_BACKGROUND,
            }
            ~{marker}
            ~{layout}
            on(action)
            Children [
                ~{Name::new("Button Text")}
                ~{text}
                TextFont { font_size: FontSize::Px(40.0) }
                TextColor(BUTTON_TEXT)
                ~{Pickable::IGNORE}
                ~{text_marker}
            ]
        ]
    }
}
