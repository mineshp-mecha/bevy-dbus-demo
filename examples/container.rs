//! Demonstrates how the to use the size constraints to control the size of a UI node.

use bevy::{
    color::palettes::css::*,
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, futures_lite::StreamExt},
};
use counter_bevy::{add_proxy::AddService, bluetooth::NotificationStream};
use regex::Regex;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_ui, setup_async_task))
        .add_systems(Update, color_change_system)
        .run();
}

// Task Resource
#[derive(Resource)]
struct AsyncTask(Task<()>);

#[derive(Component)]
struct Bar;

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
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
}

fn setup_async_task(mut commands: Commands) {
    println!("Setting up async task");

    let pool = AsyncComputeTaskPool::get();
    let (tx, rx) = mpsc::channel(10);
    commands.insert_resource(ColorReceiver { rx });

    let task = pool.spawn(async move {
        let mut stream = get_notification_stream().await.unwrap();
        println!("Notification stream started");
        while let Some(msg) = stream.next().await {
            println!("Received notification");
            let color_str = msg.args().unwrap().event;
            let parsesd_color = parse_rgb(&color_str.color).unwrap();
            match tx.send(parsesd_color).await {
                Ok(_) => (),
                Err(e) => println!("Error sending color: {}", e),
            }
        }
    });

    commands.insert_resource(AsyncTask(task));
}

#[derive(Resource)]
struct ColorReceiver {
    rx: mpsc::Receiver<BarColor>,
}

// Structure to hold the color
#[derive(Default, Debug)]
struct BarColor {
    r: f32,
    g: f32,
    b: f32,
}

fn color_change_system(
    mut query: Query<(&mut BackgroundColor, &Bar)>,
    mut receiver: ResMut<ColorReceiver>,
) {
    if let Ok(res) = receiver.rx.try_recv() {
        for (mut color, _) in query.iter_mut() {
            println!("Received color: {:?}", res);
            // Update the color of the bar
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
pub async fn get_notification_stream() -> Result<NotificationStream<'static>> {
    let stream = AddService::get_notification_stream().await?;
    Ok(stream)
}
