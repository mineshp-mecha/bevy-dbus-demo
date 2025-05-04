use zbus::{Connection, Result as ZbusResult, proxy};

use crate::AddNotificationEvent;

#[proxy(
    interface = "org.mechanix.services.Add",
    default_service = "org.mechanix.services.Add",
    default_path = "/org/mechanix/services/Add"
)]
trait Add {
    #[zbus(signal)]
    async fn notification(&self, event: AddNotificationEvent) -> ZbusResult<()>;
}

pub struct AddService;

impl AddService {
    pub async fn get_notification_stream() -> ZbusResult<NotificationStream<'static>> {
        let connection = Connection::session().await?;
        let proxy = AddProxy::new(&connection).await?;
        let stream: NotificationStream = proxy.receive_notification().await?;
        Ok(stream)
    }
}
