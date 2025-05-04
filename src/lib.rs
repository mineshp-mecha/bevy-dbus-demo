use zbus::zvariant::{DeserializeDict, SerializeDict, Type};
pub mod add_proxy;
#[derive(DeserializeDict, SerializeDict, Type, Debug)]
// `Type` treats `BluetoothNotificationEvent` is an alias for `a{sv}`.
#[zvariant(signature = "a{sv}")]
pub struct AddNotificationEvent {
    pub color: String,
}

pub mod bluetooth {
    use crate::add_proxy;
    pub use add_proxy::{AddService, NotificationStream};
}

