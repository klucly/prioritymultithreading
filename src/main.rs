use bevy::window::PresentMode;
use bevy::{asset::RenderAssetUsages, prelude::*, render::render_resource::*};
use bevy::asset::Assets;
use std::cmp::Eq;

mod sliderplugin;
mod main_controller;
mod interface;

use main_controller::MainImageData;
use crate::main_controller::MainController;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
enum ProgramState {
    #[default]
    Loading,
    Running
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: PresentMode::Immediate,
                ..default()
            }),
            ..default()
        }))
        .init_state::<ProgramState>()
        .add_plugins(sliderplugin::SliderPlugin)
        .add_systems(Startup, (setup, interface::setup_ui.after(setup)))
        .add_systems(Update, finish_loading.run_if(in_state(ProgramState::Loading)))
        .add_systems(OnEnter(ProgramState::Running), (interface::setup_sliders, main_controller_init))
        .add_systems(Update, (update, start_button_controller, stop_button_controller).run_if(in_state(ProgramState::Running)))
        .run();
}

fn finish_loading(
    main_image_data: ResMut<MainImageData>,
    images: ResMut<Assets<Image>>,
    mut asset_events: EventReader<AssetEvent<Image>>,
    mut program_state: ResMut<NextState<ProgramState>>,
) {
    for event in asset_events.read() {
        if let AssetEvent::LoadedWithDependencies { id: _ } = event {
            update_raw_ptr(images, main_image_data);
            program_state.set(ProgramState::Running);
            return;
        }
    }
}

/// NOTE: Only works for the first loaded asset. Careful
fn update_raw_ptr(mut images: ResMut<Assets<Image>>, mut main_image_data: ResMut<MainImageData>) {
    let image: &mut Image = match images.get_mut(&main_image_data.handle()) {
        Some(value) => value,
        _ => return
    };
    let img_raw_ptr = image.data.as_mut().expect("Image could not be initialized").as_mut_ptr();
    main_image_data._set_data_ptr(img_raw_ptr as usize);
}

fn setup(
    mut commands: Commands,
    server: Res<AssetServer>,
) {
    commands.spawn(Camera2d);
    let width = 1000;
    let height = 1000;

    let image = Image::new_fill(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[128, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD
    );

    let handle = server.add(image);
    commands.insert_resource(MainImageData::new(handle, width, height, 0));
}

fn update(
    main_image_data: Res<MainImageData>,
    mut images: ResMut<Assets<Image>>,
) {
    // Get an image for bevy to update it on gpu
    images.get_mut(&main_image_data.handle()).expect("Image not found");
}

fn main_controller_init(
    mut commands: Commands,
    image_data: Res<MainImageData>,
) {
    let mut main_controller = MainController::new(&image_data, 1);
    main_controller.init();

    commands.insert_resource(main_controller);
}

fn start_button_controller(
    main_controller: Res<MainController>,
    start_button: Query<(&mut BackgroundColor, &Interaction), (Changed<Interaction>, With<interface::StartButton>)>,
) {
    for (mut bg_color, interaction) in start_button {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = interface::START_BUTTON_PRESSED_COLOR.into();
                main_controller.start_all().unwrap();
            }
            Interaction::Hovered => {
                *bg_color = interface::START_BUTTON_HOVERED.into();
            }
            Interaction::None => {
                *bg_color = interface::START_BUTTON_IDLE_COLOR.into();
            }
        };
    }
}

fn stop_button_controller(
    main_controller: Res<MainController>,
    stop_button: Query<(&mut BackgroundColor, &Interaction), (Changed<Interaction>, With<interface::StopButton>)>,
) {
    for (mut bg_color, interaction) in stop_button {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = interface::STOP_BUTTON_PRESSED_COLOR.into();
                main_controller.stop_all().unwrap();
            }
            Interaction::Hovered => {
                *bg_color = interface::STOP_BUTTON_HOVERED.into();
            }
            Interaction::None => {
                *bg_color = interface::STOP_BUTTON_IDLE_COLOR.into();
            }
        };
    }
}
