use bevy::app::App;
use bevy::log;
use bevy::prelude::*;

const PI: f32 = 3.14159;

pub struct SliderPlugin;
impl Plugin for SliderPlugin {
    fn build(&self, app: &mut App) {
        log::info!("SliderPlugin initialized");
        app
            .insert_resource(SliderAnimationController::new(1., 0.5, 1.5))
            .add_systems(Update, update_sliders)
            .add_systems(PreUpdate, animate_sliders);
    }
}

#[derive(Component, Debug, Default, Clone)]
pub struct Slider {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    animation_info: SliderAnimationInfo
}

#[derive(Component, Debug, Clone)]
pub enum SliderWrapper {
    FloatSlider { slider: Slider },
    DiscreteSlider { slider: Slider, step: f32 },
}

impl SliderWrapper {
    pub fn base(&self) -> &Slider {
        match self {
            SliderWrapper::FloatSlider { slider } => slider,
            SliderWrapper::DiscreteSlider { slider, .. } => slider,
        }
    }
    pub fn base_mut(&mut self) -> &mut Slider {
        match self {
            SliderWrapper::FloatSlider { slider } => slider,
            SliderWrapper::DiscreteSlider { slider, .. } => slider,
        }
    }
}

pub fn float_slider(value: f32, min: f32, max: f32) -> impl Bundle {
    let relative_val = (value - min) / (max - min);
    let slider = SliderWrapper::FloatSlider {
        slider: Slider {
            value,
            min,
            max,
            animation_info: SliderAnimationInfo {
                prev_x: relative_val,
                dy: 0.,
                y: relative_val
            }
        },
    };
    Slider::generate_entity(slider)
}

pub fn discrete_slider(value: f32, min: f32, max: f32, step: f32) -> impl Bundle {
    let relative_val = (value - min) / (max - min);
    let slider = SliderWrapper::DiscreteSlider {
        slider: Slider {
            value,
            min,
            max,
            animation_info: SliderAnimationInfo {
                prev_x: relative_val,
                dy: 0.,
                y: relative_val
            }
        },
        step: step
    };
    Slider::generate_entity(slider)
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
    fn update_slider(&self, slider: &mut Slider, dt: f32) {
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
    sliders: Query<(&mut SliderWrapper, &Children), With<SliderWrapper>>,
    anim_controller: Res<SliderAnimationController>,
    mut slider_handles: Query<&mut Node, With<SliderHandle>>,
) {
    for (mut slider_wrapper, children) in sliders {
        let slider = slider_wrapper.base_mut();

        let mut slider_handle = slider_handles
            .get_mut(children[1])
            .expect("Couldn't find a handle in a slider");

        // TODO: Set to framerate
        anim_controller.update_slider(slider, 1./60.);
        slider_handle.left = Val::Percent(slider.animation_info.y * 100.);
    }
}

#[derive(Component, Debug, Default, Clone)]
pub struct SliderHandle;

#[derive(Component, Debug, Default, Clone)]
pub struct SliderRail;

impl Slider {
    fn generate_entity(slider_wrapper: SliderWrapper) -> impl Bundle {
        let marks: bevy::ecs::spawn::SpawnIter<Box<dyn Iterator<Item = (Node, BackgroundColor, BorderRadius)> + Send + Sync>> = match slider_wrapper {
            SliderWrapper::FloatSlider { slider: _ } => bevy::ecs::spawn::SpawnIter(Box::new([].into_iter())),
            
            SliderWrapper::DiscreteSlider { ref slider, step } => {
                let value_width = slider.max - slider.min;
                let steps = value_width / step;

                let marks_poses = (1..steps as u32)
                    .map(move |i| {i as f32 / value_width});

                let marks = marks_poses.map(|mark_pos| {(
                    Node {
                        width: Val::Px(3.),
                        height: Val::Px(20.),
                        position_type: PositionType::Absolute,
                        top: Val::Percent(0.),
                        left: Val::Percent(mark_pos * 100.),

                        ..default()
                    },
                    BackgroundColor(Color::WHITE),
                    BorderRadius::MAX,
                )});
                bevy::ecs::spawn::SpawnIter(Box::new(marks))
            }
        };

        (
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BorderRadius::all(Val::Percent(20.)),
            BackgroundColor(Color::srgb(0.211, 0.211, 0.211)),
            slider_wrapper,
            Button,

            Children::spawn((
                bevy::ecs::spawn::Spawn((
                    Node {
                        width: Val::Percent(100.),
                        height: Val::Px(5.),
                        ..default()
                    },
                    BorderRadius::MAX,
                    SliderRail,
                    BackgroundColor(Color::srgb(0.7, 0.7, 0.7))
                )),
                bevy::ecs::spawn::Spawn((
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
                )),
                marks
            ))
        )
    }
}

fn update_sliders(
    sliders: Query<(&mut SliderWrapper, &Children, &Interaction, &GlobalTransform, &ComputedNode), With<SliderWrapper>>,
    window: Query<&Window>,
) {
    let cursor_pos = match window.single().expect("Could not find a window").cursor_position() {
        Some(pos) => pos,
        _ => return
    };

    for (mut slider_wrapper, children, interaction, rail_global_transform, rail_computed_node) in sliders {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if children.len() < 2 {
            panic!("Slider has less than 2 children! It can't be updated in a ui!");
        }
        
        let pos_x = rail_global_transform.translation().x;
        let size_x = rail_computed_node.size.x;

        // Add size_x/2 because position pos_x is in the center of our element
        let relative_pos = cursor_pos.x - pos_x + size_x / 2.0;
        let clamped_relative_pos = relative_pos.clamp(0., size_x);
        let mut coef_pos = clamped_relative_pos / size_x;
        let slider = match &mut (*slider_wrapper) {
            SliderWrapper::FloatSlider { slider } => { slider }
            SliderWrapper::DiscreteSlider { slider, step } => {
                let steps = (slider.max - slider.min) / (*step);
                coef_pos = (coef_pos * steps).round()/steps;
                slider
            }
        };
        
        slider.value = slider.min.lerp(slider.max, coef_pos);
    }
}