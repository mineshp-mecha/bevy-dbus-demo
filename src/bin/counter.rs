//! This example illustrates how to create a button that changes color and text based on its
//! interaction state.

use bevy::{color::palettes::basic::*, prelude::*, winit::WinitSettings};
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Only run the app when there is user input. This will significantly reduce CPU/GPU use.
        .insert_resource(WinitSettings::desktop_app())
        .insert_resource(Counter(0))
        .add_systems(Startup, setup)
        .add_systems(Update, button_system)
        .run();
}

#[derive(Resource, Component)]
struct Counter(i32);

impl Default for Counter {
    fn default() -> Self {
        Self(0)
    }
}

#[derive(Clone, Copy, Component)]
struct CounterText;

#[derive(Component)]
enum ButtonAction {
    Increment,
    Decrement,
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
fn setup(mut commands: Commands, counter: Res<Counter>, assets: Res<AssetServer>) {
    println!("counter in setup: {}", counter.0);
    // ui camera
    commands.spawn(Camera2d);
    // Text with one section

    create_counter_text(&mut commands, &counter, &assets);

    commands
        .spawn((
            Button,
            Node {
                width: Val::Px(100.0),
                height: Val::Px(65.0),
                border: UiRect::all(Val::Px(5.0)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                top: Val::Px(150.0),
                left: Val::Px(25.0),
                ..default()
            },
            BorderColor(Color::BLACK),
            BorderRadius::MAX,
            BackgroundColor(NORMAL_BUTTON),
            ButtonAction::Decrement, 
        ))
        .with_child((
            Text::new("-"),
            TextFont {
                font: assets.load("fonts/FiraSans-Bold.ttf"),
                font_size: 33.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        ));
    commands
        .spawn((
            Button,
            Node {
                width: Val::Px(100.0),
                height: Val::Px(65.0),
                border: UiRect::all(Val::Px(5.0)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                top: Val::Px(150.0),
                left: Val::Px(120.0),
                ..default()
            },
            BorderColor(Color::BLACK),
            BorderRadius::MAX,
            BackgroundColor(NORMAL_BUTTON),
            ButtonAction::Increment, 
        ))
        .with_child((
            Text::new("+"),
            TextFont {
                font: assets.load("fonts/FiraSans-Bold.ttf"),
                font_size: 33.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        ));
}

fn create_counter_text(commands: &mut Commands, counter_value: &Counter, assets: &AssetServer) {
    commands
        .spawn((
            Button,
            Node {
                width: Val::Px(100.0),
                height: Val::Px(65.0),
                border: UiRect::all(Val::Px(5.0)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                top: Val::Px(45.0),
                left: Val::Px(70.0),
                ..default()
            },
            BorderColor(Color::BLACK),
            BorderRadius::MAX,
            BackgroundColor(NORMAL_BUTTON),
        ))
        .with_child((
            Text::new(&counter_value.0.to_string()),
            TextFont {
                font: assets.load("fonts/FiraSans-Bold.ttf"),
                font_size: 33.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            CounterText, // Mark the text component
        ));
}
fn button_system(
    mut queries: ParamSet<(
        Query<
            (
                &Interaction,
                &mut BackgroundColor,
                &mut BorderColor,
                &Children,
                Option<&ButtonAction>, // Added ButtonAction component
            ),
            (Changed<Interaction>, With<Button>),
        >,
        Query<&mut Text, With<CounterText>>,
        Query<&mut Text>,
    )>,
    mut counter: ResMut<Counter>,
) {
    
    for (interaction, _, mut border_color, _, actions) in queries.p0().iter_mut() {
        // println!("button text: {}", text.0);
        match *interaction {
            Interaction::Pressed => {
                println!("pressed");

                match actions {
                    Some(ButtonAction::Increment) => {
                        counter.0 += 1;
                    }
                    Some(ButtonAction::Decrement) => {
                        counter.0 -= 1;
                    }
                    _ => {
                        println!("no action");
                    }
                }
                border_color.0 = RED.into();
            }
            Interaction::Hovered => {
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                border_color.0 = Color::BLACK;
            }
        }
    }
    for entity in queries.p1().iter_mut() {
        let mut text = entity;
        text.0 = counter.0.to_string();
    }
}
