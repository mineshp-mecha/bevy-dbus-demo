use bevy::tasks::IoTaskPool;
use bevy::{prelude::*, winit::WinitSettings};
use freedesktop_network_manager_client::interfaces::wireless::WifiState;
use freedesktop_network_manager_client::service::NetworkManagerService;
use std::sync::mpsc::{channel, Receiver};
use std::sync::{LazyLock, Mutex};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(NetworkManagerServicePlugin)
        .insert_resource(WinitSettings::desktop_app())
        .add_systems(Startup, setup)
        .run();
}

/// Holds the async-initialized service, or None if not ready yet.
#[derive(Resource)]
pub struct NetworkManagerServiceResource {
    pub service: Option<NetworkManagerService>,
}

#[derive(Resource)]
pub struct WifiStateReceiver(pub Mutex<Receiver<WifiState>>);

#[derive(Resource, Default)]
struct WifiEventChannelInitialized(bool);
pub struct NetworkManagerServicePlugin;

impl Plugin for NetworkManagerServicePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WifiStateEvent>()
            .insert_resource(NetworkManagerServiceResource { service: None })
            .insert_resource(WifiEventChannelInitialized(false))
            .insert_resource(WifiEventText("Connected".to_string()))
            .add_systems(Startup, init_network_manager_service) // Async task so temp move service result to static
            .add_systems(Update, poll_service_init) // Once a service is initialized, it will move service from static to resource
            .add_systems(Update, setup_wifi_event_channel_async) // Async task so temp move result to static
            .add_systems(
                Update,
                (
                    poll_wifi_event_channel, // Once a channel is initialized, it will move a result from static to resource
                    enable_wifi_system.run_if(service_ready),
                    wifi_event_bridge_system.after(poll_wifi_event_channel),
                ),
            )
            .add_systems(Update, handle_wifi_state_events);
    }
}
/// Checks if the `NetworkManagerService` is ready (i.e. not None).
///
/// This is used to gate the execution of systems that depend on the service
/// being available.
fn service_ready(resource: Res<NetworkManagerServiceResource>) -> bool {
    resource.service.is_some()
}

static SERVICE_RESULT: LazyLock<Mutex<Option<NetworkManagerService>>> =
    LazyLock::new(|| Mutex::new(None));

/// This plugin provides a resource for the `NetworkManagerService` which is
/// initialized asynchronously on startup. It also provides a system for enabling
/// WiFi.
///
/// The `NetworkManagerService` is not available until the `init_network_manager_service`
/// system has completed. This is checked with the `service_ready` function.
fn init_network_manager_service() {
    IoTaskPool::get()
        .spawn(async {
            match NetworkManagerService::new().await {
                Ok(service) => {
                    let mut lock = SERVICE_RESULT.lock().unwrap();
                    *lock = Some(service);
                    info!("NetworkManagerService initialized!");
                }
                Err(e) => {
                    error!("Failed to initialize NetworkManagerService: {e}");
                }
            }
        })
        .detach();
}

// Startup system: create channel and spawn async/event producer
static WIFI_RX_RESULT: LazyLock<Mutex<Option<Receiver<WifiState>>>> =
    LazyLock::new(|| Mutex::new(None));

fn setup_wifi_event_channel_async(
    service_res: Res<NetworkManagerServiceResource>,
    mut wifi_channel_flag: ResMut<WifiEventChannelInitialized>,
) {
    if wifi_channel_flag.0 {
        // Already initialized, do nothing
        return;
    }
    if let Some(service) = &service_res.service {
        let service = service.clone();
        bevy::tasks::IoTaskPool::get()
            .spawn(async move {
                let receiver = service.subscribe_device_events().await;
                *WIFI_RX_RESULT.lock().unwrap() = Some(receiver);
            })
            .detach();
        wifi_channel_flag.0 = true; // Mark as initialized
    }
}

// Polling system to insert the resource when ready
fn poll_wifi_event_channel(mut commands: Commands) {
    let mut lock = WIFI_RX_RESULT.lock().unwrap();
    if let Some(rx) = lock.take() {
        commands.insert_resource(WifiStateReceiver(Mutex::new(rx)));
    }
}
// Polling system to move service from static to resource
fn poll_service_init(mut resource: ResMut<NetworkManagerServiceResource>) {
    let mut lock = SERVICE_RESULT.lock().unwrap();
    if let Some(service) = lock.take() {
        resource.service = Some(service);
    }
}
fn enable_wifi_system(
    resource: Res<NetworkManagerServiceResource>,
    mut queries: ParamSet<(
        Query<
            (
                &Interaction,
                Option<&ButtonAction>, // Added ButtonAction component
            ),
            (Changed<Interaction>, With<Button>),
        >,
        Query<&mut Text>,
    )>,
) {
    if let Some(service) = &resource.service {
        for (interaction, actions) in queries.p0().iter_mut() {
            // println!("button text: {}", text.0);
            match *interaction {
                Interaction::Pressed => {
                    let service = service.clone();
                    IoTaskPool::get()
                        .spawn(async move {
                            let _ = service.set_wifi(true).await;
                        })
                        .detach();
                }
                _ => {}
            }
        }
    }
}

fn wifi_event_bridge_system(
    wifi_rx: Option<Res<WifiStateReceiver>>,
    mut writer: EventWriter<WifiStateEvent>,
) {
    if let Some(wifi_rx) = wifi_rx {
        let rx = wifi_rx.0.lock().unwrap();
        while let Ok(wifi_state) = rx.try_recv() {
            writer.write(WifiStateEvent(wifi_state));
        }
    }
}

fn handle_wifi_state_events(
    mut reader: EventReader<WifiStateEvent>,
    mut query: Query<&mut Text, With<WifiStatusText>>,
) {
    for event in reader.read().into_iter() {
        info!("event reader: wifi state: {:?}", event.0);
        for mut text in query.iter_mut() {
            text.0 = event.0.to_string().clone();
        }
        // Handle logic here
    }
}

#[derive(Event, Debug, Clone)]
pub struct WifiStateEvent(pub WifiState);

#[derive(Resource, Component)]
struct WifiEventText(String);

#[derive(Clone, Copy, Component)]
struct WifiStatusText;

#[derive(Component)]
enum ButtonAction {
    Wifi,
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
fn setup(mut commands: Commands, wifi_event_text: Res<WifiEventText>, assets: Res<AssetServer>) {
    // ui camera
    commands.spawn(Camera2d);
    // Text with one section

    create_counter_text(&mut commands, &wifi_event_text, &assets);

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
            ButtonAction::Wifi,
        ))
        .with_child((
            Text::new("Wifi"),
            TextFont {
                font: assets.load("fonts/FiraSans-Bold.ttf"),
                font_size: 33.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        ));
}

fn create_counter_text(commands: &mut Commands, wifi_status: &WifiEventText, assets: &AssetServer) {
    commands
        .spawn((
            Button,
            Node {
                width: Val::Px(100.0),
                height: Val::Px(65.0),
                // border: UiRect::all(Val::Px(5.0)),
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
            Text::new(&wifi_status.0.to_string()),
            TextFont {
                font: assets.load("fonts/FiraSans-Bold.ttf"),
                font_size: 33.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            WifiStatusText, // Mark the text component
        ));
}