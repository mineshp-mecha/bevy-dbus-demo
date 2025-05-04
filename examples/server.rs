use std::time::Duration;

use counter_bevy::AddNotificationEvent;
use tokio::time;
use zbus::{
    connection,
    fdo::Error as ZbusError,
    interface,
    object_server::SignalContext
};

#[derive(Clone, Copy)]
pub struct AddBusInterface {}

#[interface(name = "org.mechanix.services.Add")]
impl AddBusInterface {
    pub async fn add_number(&self, number1: i8, number2: i8) -> Result<i8, ZbusError> {
        let result = number1 + number2;
        println!("Adding {} + {} = {}", number1, number2, result);
        Ok(result)
    }

    #[zbus(signal)]
    async fn notification(
        &self,
        ctxt: &SignalContext<'_>,
        event: AddNotificationEvent,
    ) -> Result<(), zbus::Error>;
}

#[tokio::main]
async fn main() {
    let bus = AddBusInterface {};
    let bus_connection = connection::Builder::session()
        .unwrap()
        .name("org.mechanix.services.Add")
        .unwrap()
        .serve_at("/org/mechanix/services/Add", bus)
        .unwrap()
        .build()
        .await
        .unwrap();

    let handler = tokio::spawn(async move {
        if let Err(e) =
            event_notification_stream(&bus, &bus_connection).await
        {
            println!("Error notification stream: {}", e);
        }
    });
    match handler.await {
        Ok(_) => println!("handler await"),
        Err(err) => println!("error:{}", err),
    }
}

pub async fn event_notification_stream(
    add_bus: &AddBusInterface,
    conn: &zbus::Connection,
) -> Result<(), ZbusError> {
    let mut interval = time::interval(Duration::from_secs(01));
    loop {
        interval.tick().await;
        // Generate a random color
        let r: f32 = fastrand::f32();
        let g: f32 = fastrand::f32();
        let b: f32 = fastrand::f32();
        let color_str = format!("RGB({}, {}, {})", r, g, b);
        // Send signal if there's a change in status
        let ctxt = match SignalContext::new(conn, "/org/mechanix/services/Add") {
            Ok(ctxt) => ctxt,
            Err(e) => {
                println!("Error creating signal context: {}", e);
                continue;
            }
        };

        match add_bus
            .notification(&ctxt, AddNotificationEvent { color: color_str })
            .await
        {
            Ok(_) => {
                println!("Notification sent");
            }
            Err(e) => {
                println!("Error sending notification: {}", e);
            }
        }
    }
}
