//! Demonstrates how the to use the size constraints to control the size of a UI node.

use std::{
    thread,
    time::{self, Duration},
};

use bevy::{color::palettes::css::*, prelude::*};
use regex::Regex;
use tokio::{sync::mpsc, task};
use zmq::SocketType;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        // .add_systems(Update, color_change_system)
        .run();
}

#[derive(Component)]
struct Bar;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // ui camera
    commands.spawn(Camera2d);

    let text_font = (
        TextFont {
            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size: 33.0,
            ..Default::default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
    );

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Click a button to see the effect of the size constraints"),
                        text_font.clone(),
                        Node {
                            margin: UiRect::bottom(Val::Px(25.)),
                            ..Default::default()
                        },
                    ));

                    // spawn_bar(parent);
                    parent
                        .spawn((
                            Node {
                                flex_basis: Val::Percent(100.0),
                                align_self: AlignSelf::Stretch,
                                padding: UiRect::all(Val::Px(10.)),
                                ..default()
                            },
                            BackgroundColor(YELLOW.into()),
                        ))
                        .with_children(|parent| {
                            parent
                                .spawn((
                                    Node {
                                        align_items: AlignItems::Stretch,
                                        width: Val::Percent(100.),
                                        height: Val::Px(100.),
                                        padding: UiRect::all(Val::Px(4.)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::WHITE),
                                    Bar,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((Node::default(), BackgroundColor(RED.into())));
                                });
                        });
                });
        });

    let (tx, rx) = mpsc::channel(10);
    commands.insert_resource(ColorReceiver { rx });
    
    // Start the ZeroMQ client task
    thread::spawn(move || {
        let context = zmq::Context::new();
        let requester = context.socket(SocketType::REQ).unwrap();

        assert!(requester.connect("tcp://localhost:5555").is_ok());
        println!("Connection made successfully");
        loop {
            // ...

            // Receive the color from the server
            let mut msg = zmq::Message::new();
            requester.send("Hello", 0).unwrap();
            requester.recv(&mut msg, 0).unwrap();
            let color_str = msg.as_str().unwrap();
            println!("Received color: {}", color_str);
            if let Ok(bar_colors) = parse_rgb(color_str) {
               
                // Send the color to the main thread via the channel
                tx.blocking_send(bar_colors).unwrap();
            };

            // Wait a bit before sending the next request
            // thread::sleep(Duration::from_millis(1000));
        }
    });
}

#[derive(Resource)]
struct ColorReceiver {
    rx: mpsc::Receiver<BarColor>,
}

// Structure to hold the color
#[derive(Default)]
struct BarColor {
    r: f32,
    g: f32,
    b: f32,
}

fn color_change_system(
    mut query: Query<(&mut BackgroundColor, &Bar)>,
    mut receiver: ResMut<ColorReceiver>,
) {
    println!("inside update");
    if let Ok(res) = receiver.rx.try_recv() {
        for (mut color, _) in query.iter_mut() {
            color.0 = Color::srgb(res.r, res.g, res.b);
        }
    }
}

fn parse_rgb(s: &str) -> Result<BarColor, String> {
    let re = Regex::new(r"(\d+(?:\.\d+)?)").unwrap();
    let captures: Vec<f32> = re
        .find_iter(s)
        .map(|m| m.as_str().parse::<f32>().unwrap())
        .collect();

    if captures.len() != 3 {
        return Err("Invalid RGB format".to_string());
    }

    Ok(BarColor {
        r: captures[0],
        g: captures[1],
        b: captures[2],
    })
}
