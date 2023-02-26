use zbus::{dbus_interface, Connection};

struct TimeDate {
    name: String,
}

#[dbus_interface(name = "org.freedesktop.timedate1")]
impl TimeDate {
    async fn set_time(&self, usec_utc: i64, relative: bool, interactive: bool) {
        ()
    }

    async fn set_timezone(&self, timezone: String, interactive: bool) {
        ()
    }

    #[dbus_interface(name = "SetLocalRTC")]
    async fn set_local_rtc(&self, local_rtc: bool, fix_system: bool, interactive: bool) {
        ()
    }

    #[dbus_interface(name = "SetNTP")]
    async fn set_ntp(&self, use_ntp: bool, interactive: bool) {
        ()
    }

    async fn list_timezones(&self) -> Vec<String> {
        vec![String::new()]
    }

    #[dbus_interface(property)]
    async fn timezone(&self) -> &str {
        &self.name
    }

    #[dbus_interface(property, name = "LocalRTC")]
    async fn local_rtc(&self) -> bool {
        true
    }

    #[dbus_interface(property, name = "CanNTP")]
    async fn can_ntp(&self) -> bool {
        true
    }

    #[dbus_interface(property, name = "NTP")]
    async fn ntp(&self) -> bool {
        true
    }

    #[dbus_interface(property, name = "NTPSynchronized")]
    async fn ntp_synchronized(&self) -> bool {
        true
    }

    #[dbus_interface(property, name = "TimeUSec")]
    async fn time_usec(&self) -> u64 {
        0
    }

    #[dbus_interface(property, name = "RTCTimeUSec")]
    async fn rtc_time_usec(&self) -> u64 {
        0
    }
}

#[tokio::main]
async fn main() -> zbus::Result<()> {
    let timedate = TimeDate {
        name: "foo".into(),
    };
    let conn = Connection::session().await?;
    conn.object_server().at("/org/freedesktop/timedate1", timedate).await?;
    conn.request_name("org.freedesktop.timedate1").await?;

    loop {
        std::future::pending::<()>().await;
    }
}
