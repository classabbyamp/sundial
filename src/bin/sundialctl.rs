use zbus::{dbus_proxy, Connection};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::system().await?;
    let proxy = timedate1Proxy::new(&conn).await?;

    let time = proxy.time_usec().await?;
    let tz = proxy.timezone().await?;
    let rtctime = proxy.rtctime_usec().await;
    let localrtc = proxy.local_rtc().await?;

    println!("System time: {}", time);
    println!("System timezone: {}", tz);
    println!("RTC time: {:?}", rtctime);
    println!("RTC timezone: {}", if localrtc { "Local" } else { "UTC"} );
    Ok(())
}

#[dbus_proxy(
    interface = "org.freedesktop.timedate1",
    default_service = "org.freedesktop.timedate1",
    default_path = "/org/freedesktop/timedate1"
)]
trait timedate1 {
    fn list_timezones(&self) -> zbus::Result<Vec<String>>;

    #[dbus_proxy(name = "SetLocalRTC")]
    fn set_local_rtc(
        &self,
        local_rtc: bool,
        fix_system: bool,
        interactive: bool,
    ) -> zbus::Result<()>;

    // #[dbus_proxy(name = "SetNTP")]
    // fn set_ntp(&self, use_ntp: bool, interactive: bool) -> zbus::Result<()>;

    fn set_time(&self, usec_utc: i64, relative: bool, interactive: bool) -> zbus::Result<()>;

    fn set_timezone(&self, timezone: &str, interactive: bool) -> zbus::Result<()>;

    // #[dbus_proxy(property, name = "CanNTP")]
    // fn can_ntp(&self) -> zbus::Result<bool>;

    #[dbus_proxy(property, name = "LocalRTC")]
    fn local_rtc(&self) -> zbus::Result<bool>;

    // #[dbus_proxy(property, name = "NTP")]
    // fn ntp(&self) -> zbus::Result<bool>;

    #[dbus_proxy(property, name = "NTPSynchronized")]
    fn ntpsynchronized(&self) -> zbus::Result<bool>;

    #[dbus_proxy(property, name = "RTCTimeUSec")]
    fn rtctime_usec(&self) -> zbus::Result<u64>;

    #[dbus_proxy(property, name = "TimeUSec")]
    fn time_usec(&self) -> zbus::Result<u64>;

    #[dbus_proxy(property)]
    fn timezone(&self) -> zbus::Result<String>;
}
