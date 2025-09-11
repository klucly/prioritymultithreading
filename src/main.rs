use bevy::window::PresentMode;
use bevy::{asset::RenderAssetUsages, prelude::*, render::render_resource::*};
use bevy::asset::Assets;
use std::thread;
use rand::Rng;

mod sliderplugin;
mod main_program;
mod interface;

use main_program::MainImageData;
use crate::main_program::MainProgram;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: PresentMode::Immediate,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(sliderplugin::SliderPlugin)
        .add_systems(Startup, (setup, interface::setup_ui.after(setup)))
        .add_systems(Update, (update, setup_img_raw_ptr, interface::setup_sliders, main_program_init.after(setup_img_raw_ptr)))
        .run();
}

fn setup_img_raw_ptr(
    mut main_image_data: ResMut<MainImageData>,
    mut images: ResMut<Assets<Image>>,
    mut asset_events: EventReader<AssetEvent<Image>>,
) {
    for event in asset_events.read() {
        if let AssetEvent::LoadedWithDependencies { id: _id } = event {
            let image: &mut Image = match images.get_mut(&main_image_data.handle()) {
                Some(value) => value,
                _ => return
            };
            let img_raw_ptr = image.data.as_mut().expect("Image could not be initialized").as_mut_ptr();
            main_image_data._set_data_ptr(img_raw_ptr as usize);
        }
    }
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

fn random_color() -> (u8, u8, u8, u8) {
    let mut rng = rand::rng();
    (
        rng.random_range(0..=255),
        rng.random_range(0..=255),
        rng.random_range(0..=255),
        255
    )
}

fn update(
    main_image_data: Res<MainImageData>,
    mut images: ResMut<Assets<Image>>,
    keycode: Res<ButtonInput<KeyCode>>,
) {
    // Get an image for bevy to update it on gpu
    images.get_mut(&main_image_data.handle()).expect("Image not found");

    if !keycode.just_pressed(KeyCode::Space) {
        return
    }
    let width = main_image_data.width();
    let height = main_image_data.height();
    let raw_img_ptr = main_image_data.data_ptr();
    let color = random_color();
    thread::spawn(move || unsafe {
        let img_ptr = raw_img_ptr as *mut u8;
        loop {
            for x in 0..width {
                for y in 0..height {
                    let index = 4 * (x + y * width) as usize;
                    *img_ptr.add(index+0) = color.0;
                    *img_ptr.add(index+1) = color.1;
                    *img_ptr.add(index+2) = color.2;
                    *img_ptr.add(index+3) = color.3;
                }
            }
        }
    });
}

fn main_program_init(
    image_data: Res<MainImageData>,
) {
    MainProgram::init(&image_data, 1);
}
