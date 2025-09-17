use bevy::prelude::*;
use std::thread;

use crate::sliderplugin;
use crate::main_controller;

#[derive(Component, Debug, Default, Clone)]
pub struct StartButton;

#[derive(Component, Debug, Default, Clone)]
pub struct StopButton;

pub const START_BUTTON_IDLE_COLOR: Color = Color::srgb(0.20, 0.35, 0.25);
pub const START_BUTTON_PRESSED_COLOR: Color = Color::srgb(0.20*0.5, 0.35*0.5, 0.25*0.5);
pub const START_BUTTON_HOVERED: Color = Color::srgb(0.20, 0.35*2., 0.25);

pub const STOP_BUTTON_IDLE_COLOR: Color = Color::srgb(0.35, 0.20, 0.20);
pub const STOP_BUTTON_PRESSED_COLOR: Color = Color::srgb(0.35*0.5, 0.20*0.5, 0.20*0.5);
pub const STOP_BUTTON_HOVERED: Color = Color::srgb(0.35*2., 0.20, 0.20);

pub fn setup_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    main_image_data: Res<main_controller::MainImageData>
) {
    let start_button = (
        StartButton,
        Button,
        Node {
            width: Val::Percent(50.0),
            height: Val::Percent(50.0),
            margin: UiRect::all(Val::Percent(5.)),

            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BorderRadius::percent(20., 20., 20., 20.),
        BackgroundColor(START_BUTTON_IDLE_COLOR),
        children![(
            Text::new("Start"),
            TextFont {
                font: asset_server.load("Inter-Black.ttf"),
                font_size: 33.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        )]
    );
    let stop_button = (
        StopButton,
        Button,
        Node {
            width: Val::Percent(50.0),
            height: Val::Percent(50.0),
            margin: UiRect::all(Val::Percent(5.)),

            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor(Color::BLACK),
        BorderRadius::percent(20., 20., 20., 20.),
        BackgroundColor(STOP_BUTTON_IDLE_COLOR),
        children![(
            Text::new("Stop"),
            TextFont {
                font: asset_server.load("Inter-Black.ttf"),
                font_size: 33.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        )]
    );
    let buttons_frame = (
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(100.0),

            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            start_button,
            stop_button
        ]
    );
    let options_header_frame = (
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(60.0),
            bottom: Val::Px(5.0),

            ..default()
        },
        BackgroundColor(Color::srgb(0.16, 0.16, 0.18)),
    );
    let sliders_main_frame = (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),

            flex_direction: FlexDirection::Column,

            ..default()
        },
        BackgroundColor(Color::srgb(0.16, 0.16, 0.18)),
        Sliders,
    );

    let options_main_frame = (
        Node {
            width: Val::Percent(40.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,

            ..default()
        },
        children![
            options_header_frame,
            sliders_main_frame,
            buttons_frame
        ]
    );
    let screen_frame = (
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(60.),
            ..default()
        },
        ImageNode {
            image: main_image_data.handle(),
            ..default()
        },
        BorderRadius::percent(10., 10., 10., 10.),
        BackgroundColor(Color::BLACK),
    );
    let process_info_frame = (
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(40.),
            ..default()
        },
        BorderRadius::percent(10., 10., 10., 10.),
    );
    let showcase_frame = (
        Node {
            width: Val::Percent(99.0),
            height: Val::Percent(99.0),
            flex_direction: FlexDirection::Column,

            ..default()
        },
        BorderRadius::percent(5., 5., 5., 5.),
        BackgroundColor(Color::srgb(0.10, 0.10, 0.12)),
        children![screen_frame, process_info_frame]
    );
    let outer_showcase_frame = (
        Node {
            width: Val::Percent(60.0),
            height: Val::Percent(100.0),
            
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            
            ..default()
        },
        children![showcase_frame]
    );
    let main_frame = (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),

            flex_direction: FlexDirection::Row,
            ..default()
        },
        children![
            options_main_frame,
            outer_showcase_frame
        ]
    );
    let title_bar = (
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(30.0),

            flex_direction: FlexDirection::Row,
            ..default()
        },
    );
    let global_frame = (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),

            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgb(0.12, 0.12, 0.14)),
        children![
            title_bar,
            main_frame,
        ]

    );

    commands.spawn(global_frame);
}


#[derive(Component)]
pub struct Sliders;


pub fn setup_sliders(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &Sliders), Added<Sliders>>,
) {
    let threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    for (entity, _slider) in query {
        commands.entity(entity).with_children(|slider_main| {
            for slider_index in 0..threads/4 {
                slider_main.spawn((
                    Node {
                        height: Val::Px(100.),
                        margin: UiRect::all(Val::Px(5.)),
                        flex_direction: FlexDirection::Row,

                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.17)),
                    BorderRadius::all(Val::Px(10.)),
                    children![
                        (
                            Node {
                                width: Val::Px(100.),
                                
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,

                                ..default()
                            },
                            children![(
                                Text::new(format!["G{}", slider_index]),
                                TextFont {
                                    font: asset_server.load("Inter-Black.ttf"),
                                    font_size: 40.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                            )]
                        ),
                        // sliderplugin::float_slider(0., -2., 2.)
                        sliderplugin::discrete_slider(1., -2., 2., 1.)
                    ]
                ));
            }
        });
    }
}
