use anyhow::Result;
use parse_datetime::parse_datetime;
use zbus::dbus_proxy;

use crate::cli::{RtcMode, RtcSyncFrom};

#[dbus_proxy(
    interface = "org.freedesktop.timedate1",
    default_service = "org.freedesktop.timedate1",
    default_path = "/org/freedesktop/timedate1"
)]
trait timedate1 {
    fn list_timezones(&self) -> zbus::Result<Vec<String>>;

    #[dbus_proxy(name = "SetLocalRTC", allow_interactive_auth)]
    fn set_local_rtc(
        &self,
        local_rtc: bool,
        fix_system: bool,
        interactive: bool,
    ) -> zbus::Result<()>;

    // #[dbus_proxy(name = "SetNTP")]
    // fn set_ntp(&self, use_ntp: bool, interactive: bool) -> zbus::Result<()>;

    #[dbus_proxy(allow_interactive_auth)]
    fn set_time(&self, usec_utc: i64, relative: bool, interactive: bool) -> zbus::Result<()>;

    #[dbus_proxy(allow_interactive_auth)]
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

impl timedate1Proxy<'_> {
    pub(crate) async fn status_cmd(&self, pretty: bool) -> Result<()> {
        let time = self.time_usec().await?;
        let tz = self.timezone().await?;
        let ntpsync = self.ntpsynchronized().await?;
        let rtctime = self.rtctime_usec().await?;
        let localrtc = self.local_rtc().await?;

        if pretty {
            println!("System time: {}", time);
            println!("System timezone: {}", tz);
            println!("NTP Synchronized: {}", if ntpsync { "yes" } else { "no" });
            println!("RTC time: {}", rtctime);
            println!("RTC timezone: {}", if localrtc { "Local" } else { "UTC" });
        } else {
            todo!()
        }
        Ok(())
    }

    pub(crate) async fn set_time_cmd(&self, time: String, interactive: bool) -> Result<()> {
        let tm = parse_datetime(time.as_str())?;
        self.set_time(tm.timestamp_micros(), false, interactive)
            .await?;
        Ok(())
    }

    pub(crate) async fn list_tz_cmd(&self) -> Result<()> {
        for tz in self.list_timezones().await? {
            println!("{tz}")
        }
        Ok(())
    }

    pub(crate) async fn set_tz_cmd(&self, timezone: String, interactive: bool) -> Result<()> {
        self.set_timezone(timezone.as_str(), interactive).await?;
        Ok(())
    }

    pub(crate) async fn set_rtc_cmd(
        &self,
        mode: RtcMode,
        sync_from: RtcSyncFrom,
        interactive: bool,
    ) -> Result<()> {
        self.set_local_rtc(mode.into(), sync_from.into(), interactive)
            .await?;
        Ok(())
    }
}
