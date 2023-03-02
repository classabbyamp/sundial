use std::io::{self, BufRead};
use std::path::Path;

use tokio::fs::{canonicalize, File};
use zbus::{dbus_interface, Connection, fdo};

struct TimeDate {
    tz: Option<String>,
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

    /// returns a list of time zones known on the local system
    async fn list_timezones(&self) -> fdo::Result<Vec<String>> {
        let fp = Path::new("/usr/share/zoneinfo/zone.tab");
        if fp.exists() {
            match read_lines(fp).await {
                Err(e) => return Err(fdo::Error::Failed(format!("Couldn't get timezone list: {}", e))),
                Ok(lines) => {
                    let mut out = vec![];
                    for ln in lines {
                        if let Ok(l) = ln {
                            // if not a comment
                            if ! l.starts_with('#') {
                                let mut parts = l.split_whitespace();
                                // grab the 3rd part of the line
                                if let Some(tz) = parts.nth(2) {
                                    out.push(tz.to_string());
                                }
                            }
                        }
                    }
                    return Ok(out);
                }
            }
        }
        Err(fdo::Error::Failed("Couldn't get timezone list: file not found".into()))
    }

    /// shows the currently configured time zone
    #[dbus_interface(property)]
    async fn timezone(&self) -> fdo::Result<String> {
        // see hwclock(8)
        if let Some(tz) = &self.tz {
            return Ok(tz.to_string());
        }

        match canonicalize("/etc/localtime").await {
            Ok(p) => match p.strip_prefix("/usr/share/zoneinfo/") {
                Ok(tz) => Ok(tz.to_string_lossy().to_string()),
                Err(_) => Err(fdo::Error::Failed("Unable to determine local timezone".into())),
            },
            // /etc/localtime doesn't exist -> assume UTC
            Err(_) => Ok("UTC".into()),
        }
    }

    /// shows whether the RTC is configured to use UTC (`false`)
    /// or the local time zone (`true`)
    #[dbus_interface(property, name = "LocalRTC")]
    async fn local_rtc(&self) -> fdo::Result<bool> {
        // see adjtime_config(5)
        // if /etc/adjtime exists, check 3rd line for "LOCAL" or "UTC"
        let fp = Path::new("/etc/adjtime");
        if fp.exists() {
            match read_lines(fp).await {
                Err(e) => return Err(fdo::Error::Failed(format!("Couldn't get RTC status: {}", e))),
                Ok(mut lines) => {
                    if let Some(Ok(ln)) = lines.nth(2) {
                        // return true if 3rd line is LOCAL, false if UTC (or otherwise)
                        return Ok(ln == "LOCAL")
                    } else {
                        // assume UTC if no 3rd line
                        return Ok(false)
                    }
                }
            }
        }
        // else assume "UTC"
        Ok(false)
    }

    #[dbus_interface(property(emits_changed_signal = "false"), name = "CanNTP")]
    async fn can_ntp(&self) -> bool {
        // TODO: this is more advanced, and probably requires some distro-specific stuff
        false
    }

    #[dbus_interface(property, name = "NTP")]
    async fn ntp(&self) -> bool {
        // TODO: this is more advanced, and probably requires some distro-specific stuff
        false
    }

    #[dbus_interface(property(emits_changed_signal = "false"), name = "NTPSynchronized")]
    async fn ntp_synchronized(&self) -> bool {
        // TODO
        false
    }

    #[dbus_interface(property(emits_changed_signal = "false"), name = "TimeUSec")]
    async fn time_usec(&self) -> u64 {
        // TODO
        0
    }

    #[dbus_interface(property(emits_changed_signal = "false"), name = "RTCTimeUSec")]
    async fn rtc_time_usec(&self) -> u64 {
        // TODO
        0
    }
}

#[tokio::main]
async fn main() -> zbus::Result<()> {
    let timedate = TimeDate {
        tz: std::env::var("TZ").ok(),
    };
    let conn = Connection::session().await?;
    conn.object_server().at("/org/freedesktop/timedate1", timedate).await?;
    conn.request_name("org.freedesktop.timedate1").await?;

    loop {
        std::future::pending::<()>().await;
    }
}

async fn read_lines<P: AsRef<Path>>(fp: P) -> io::Result<io::Lines<io::BufReader<std::fs::File>>> {
    let f = File::open(fp).await?.into_std().await;
    // TODO: use async
    Ok(io::BufReader::new(f).lines())
}
