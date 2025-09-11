use bevy::app::App;
use bevy::log;
use bevy::prelude::*;
// use bevy::animation::*;


pub struct SliderPlugin;
impl Plugin for SliderPlugin {
    fn build(&self, app: &mut App) {
        log::info!("SliderPlugin initialized");
        // app.add_systems(Startup, init_animation);
        app.add_systems(Update, update_sliders);
    }
}


// fn init_animation() {
//     let mut curve = AnimationCurve::<Vec2>::new(1);
//     curve.set_keyframe(0, Vec2::ZERO, EaseFunction::CubicOut);
//     let clip = AnimationClip::new(curve);
//     let target = AnimationTarget::property("style::left").to(Val::Px(200.0));

//     // Create the animation graph
//     let graph = AnimationGraph::from_clip(clip);
//     let player = commands.spawn(AnimationPlayer::new(graph)).id();
// }

#[derive(Component, Debug, Default, Clone)]
pub struct FloatSlider {
    pub value: f32,
    pub min: f32,
    pub max: f32,
}

#[derive(Component, Debug, Default, Clone)]
pub struct SliderHandle;

#[derive(Component, Debug, Default, Clone)]
pub struct SliderRail;

impl FloatSlider {
    pub fn create(value: f32, min: f32, max: f32) -> impl Bundle {
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
            FloatSlider { value, min, max },
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

pub fn update_sliders(
    sliders: Query<(&mut FloatSlider, &Children, &Interaction), With<FloatSlider>>,
    mut slider_handles: Query<&mut Node, With<SliderHandle>>,
    mut slider_rails: Query<(&GlobalTransform, &ComputedNode), With<SliderRail>>,
    window: Query<&Window>,
) {
    let cursor_pos = match window.single().expect("Could not find a window").cursor_position() {
        Some(pos) => pos,
        _ => return
    };

    for (mut slider_info, children, interaction) in sliders {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if children.len() < 2 {
            panic!("Slider has less than 2 children! It can't be updated in a ui!");
        }

        let (rail_global_transform, rail_computed_node) = slider_rails
            .get_mut(children[0])
            .expect("Couldn't find a rail in a slider");

        let mut slider_handle = slider_handles
            .get_mut(children[1])
            .expect("Couldn't find a handle in a slider");

        let pos_x = rail_global_transform.translation().x;
        let size_x = rail_computed_node.size.x;

        let relative_pos = cursor_pos.x - pos_x + size_x / 2.0;
        let clamped_relative_pos = relative_pos.clamp(0., size_x);
        
        slider_info.value = slider_info.min.lerp(slider_info.max, clamped_relative_pos / size_x);
        slider_handle.left = Val::Px(clamped_relative_pos - 5.);
    }
}