use bevy::app::App;
use bevy::log;
use bevy::prelude::*;

const PI: f32 = 3.14159;

pub struct SliderPlugin;
impl Plugin for SliderPlugin {
    fn build(&self, app: &mut App) {
        log::info!("SliderPlugin initialized");
        app
            .insert_resource(SliderAnimationController::new(2., 1., 1.5))
            .add_systems(Update, update_sliders)
            .add_systems(PreUpdate, animate_sliders);
    }
}

#[derive(Component, Debug, Default, Clone)]
pub struct FloatSlider {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    animation_info: SliderAnimationInfo
}

#[derive(Debug, Default, Clone)]
struct SliderAnimationInfo {
    prev_x: f32,
    y: f32,
    dy: f32,
}

#[derive(Resource)]
struct SliderAnimationController {
    k1: f32,
    k2: f32,
    k3: f32
}

impl SliderAnimationController {
    fn new(f: f32, z: f32, r: f32) -> SliderAnimationController {
        SliderAnimationController {
            k1: z / (PI * f),
            k2: 1./ ((2. * PI * f) * (2. * PI * f)),
            k3: r * z / (2. * PI * f)
        }
    }
    fn update_slider(&self, slider: &mut FloatSlider, dt: f32) {
        let x = (slider.value - slider.min) / (slider.max - slider.min);
        let prev_x = slider.animation_info.prev_x;
        let dx = (x - prev_x) / dt;
        let mut y = slider.animation_info.y;
        let mut dy = slider.animation_info.dy;
        
        if y < 0. || y > 1. {
            dy = -dy;
            y = y.clamp(0. + 0.00001, 1. - 0.00001)
        }
        slider.animation_info.y = y + dt * dy;
        slider.animation_info.dy = dy + dt * (x + self.k3*dx - y - self.k1*dy) / self.k2;
        slider.animation_info.prev_x = x;
    }
}

fn animate_sliders(
    sliders: Query<(&mut FloatSlider, &Children), With<FloatSlider>>,
    anim_controller: Res<SliderAnimationController>,
    mut slider_handles: Query<&mut Node, With<SliderHandle>>,
) {
    for (mut slider, children) in sliders {

        let mut slider_handle = slider_handles
            .get_mut(children[1])
            .expect("Couldn't find a handle in a slider");

        anim_controller.update_slider(&mut slider, 1./60.);
        slider_handle.left = Val::Percent(slider.animation_info.y * 100.);
    }
}

#[derive(Component, Debug, Default, Clone)]
pub struct SliderHandle;

#[derive(Component, Debug, Default, Clone)]
pub struct SliderRail;

impl FloatSlider {
    pub fn create(value: f32, min: f32, max: f32) -> impl Bundle {
        let relative_val = (value - min) / (max - min);
        (
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BorderRadius::MAX,
            BackgroundColor(Color::srgb(0.211, 0.211, 0.211)),
            FloatSlider { value, min, max, animation_info: SliderAnimationInfo { prev_x: relative_val, dy: 0., y: relative_val } },
            Button,

            children![
                (
                    Node {
                        width: Val::Percent(100.),
                        height: Val::Px(5.),
                        ..default()
                    },
                    BorderRadius::MAX,
                    SliderRail,
                    BackgroundColor(Color::srgb(0.7, 0.7, 0.7))
                ),
                (
                    Node {
                        width: Val::Px(10.),
                        height: Val::Px(30.),
                        left: Val::Percent(75.),
                        position_type: PositionType::Absolute,
                        
                        ..default()
                    },
                    SliderHandle,
                    BorderRadius::MAX,
                    BackgroundColor(Color::srgb(0.7, 0.7, 0.7))
                )
            ]
        )
    }
}

fn update_sliders(
    sliders: Query<(&mut FloatSlider, &Children, &Interaction, &GlobalTransform, &ComputedNode), With<FloatSlider>>,
    window: Query<&Window>,
) {
    let cursor_pos = match window.single().expect("Could not find a window").cursor_position() {
        Some(pos) => pos,
        _ => return
    };

    for (mut slider_info, children, interaction, rail_global_transform, rail_computed_node) in sliders {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if children.len() < 2 {
            panic!("Slider has less than 2 children! It can't be updated in a ui!");
        }
        
        let pos_x = rail_global_transform.translation().x;
        let size_x = rail_computed_node.size.x;

        let relative_pos = cursor_pos.x - pos_x + size_x / 2.0;
        let clamped_relative_pos = relative_pos.clamp(0., size_x);
        
        slider_info.value = slider_info.min.lerp(slider_info.max, clamped_relative_pos / size_x);
    }
}