//! Demonstrates how the to use the size constraints to control the size of a UI node.

use bevy::{color::palettes::css::*, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Tic Tac Toe".into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, button_click)
        .run();
}

#[derive(Component)]
struct Position {
    row: usize,
    col: usize,
}
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let text_font = (
        TextFont {
            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size: 33.0,
            ..Default::default()
        },
        TextColor::BLACK,
    );
    commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            BackgroundColor(Color::WHITE),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Tic Tac Toe"),
                text_font.clone(),
                Node {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..Default::default()
                },
            ));

            // Game Board
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    BackgroundColor(GRAY.into()),
                ))
                .with_children(|parent| {
                    for _row in 0..3 {
                        parent
                            .spawn((
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..Default::default()
                                },
                                BackgroundColor(GRAY.into()),
                            ))
                            .with_children(|column| {
                                for col in 0..3 {
                                    column.spawn((
                                        Button,
                                        Node {
                                            width: Val::Px(100.0),
                                            height: Val::Px(100.0),
                                            margin: UiRect {
                                                left: Val::Px(5.0),
                                                right: Val::Px(5.0),
                                                top: Val::Px(5.0),
                                                bottom: Val::Px(5.0),
                                            },
                                            ..Default::default()
                                        },
                                        BackgroundColor(YELLOW.into()),
                                    ));
                                }
                            });
                    }
                });
        });
}

fn button_click(
    mut interaction_query: Query<(&Interaction, &Position), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, position) in &mut interaction_query {
        match *interaction {
            Interaction::Hovered => {
                println!("Hovered button at ({}, {})", position.row, position.col);
            }
            Interaction::Pressed => {
                println!("Pressed button at ({}, {})", position.row, position.col);
            }
            Interaction::None => {
                println!("None button at ({}, {})", position.row, position.col);
            }
        }
    }
}
