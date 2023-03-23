use std::{path::Path, os::fd::RawFd};

use nix::errno::Errno;
use tokio::fs::canonicalize;
use zbus::{dbus_interface, Connection, fdo};

use util::{rtc_open, rtc_read, read_lines};

mod util;

const SEC_TO_USEC: libc::c_long = 1_000_000;
const NSEC_TO_USEC: libc::c_long = 1_000;
const MAX_PHASE: libc::c_long = 500_000_000;

struct TimeDate {
    tz: Option<String>,
    rtc: Result<RawFd, Errno>,
}

#[dbus_interface(name = "org.freedesktop.timedate1")]
impl TimeDate {
    /// change the system clock
    #[allow(unused_variables)]
    async fn set_time(&self, usec_utc: i64, relative: bool, interactive: bool) -> fdo::Result<()> {
        // TODO
        // https://github.com/systemd/systemd/blob/main/src/timedate/timedated.c#L820
        // TODO: if ntp enabled, fail

        if relative {
            if usec_utc == 0 {
                return Ok(());
            }


        } else {
            if usec_utc <= 0 {
                return Err(fdo::Error::InvalidArgs("Invalid absolute time".into()));
            }
        }

        Ok(())
    }

    /// set the system timezone
    #[allow(unused_variables)]
    async fn set_timezone(&self, timezone: String, interactive: bool) -> fdo::Result<()> {
        // TODO
        // https://github.com/systemd/systemd/blob/main/src/timedate/timedated.c#L657
        Ok(())
    }

    /// control whether the RTC is in local time or UTC
    #[dbus_interface(name = "SetLocalRTC")]
    #[allow(unused_variables)]
    async fn set_local_rtc(&self, local_rtc: bool, fix_system: bool, interactive: bool) -> fdo::Result<()> {
        // TODO
        // https://github.com/systemd/systemd/blob/main/src/timedate/timedated.c#L734
        Ok(())
    }

    /// control whether the system clock is synchronized with the network
    #[dbus_interface(name = "SetNTP")]
    #[allow(unused_variables)]
    async fn set_ntp(&self, use_ntp: bool, interactive: bool) -> fdo::Result<()> {
        Err(fdo::Error::NotSupported("NTP is not supported".into()))
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

    /// shows whether the RTC is configured to use UTC (`false`) or the local time zone (`true`)
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
                        return Ok(ln == "LOCAL");
                    }
                }
            }
        }
        // else assume UTC
        Ok(false)
    }

    /// shows whether a service to perform time synchronization over the network is available
    #[dbus_interface(property(emits_changed_signal = "false"), name = "CanNTP")]
    async fn can_ntp(&self) -> fdo::Result<bool> {
        Err(fdo::Error::NotSupported("NTP is not supported".into()))
    }

    /// shows whether a service to perform time synchronization over the network is enabled
    #[dbus_interface(property, name = "NTP")]
    async fn ntp(&self) -> fdo::Result<bool> {
        Err(fdo::Error::NotSupported("NTP is not supported".into()))
    }

    /// shows whether the kernel reports the time as synchronized
    #[dbus_interface(property(emits_changed_signal = "false"), name = "NTPSynchronized")]
    async fn ntp_synchronized(&self) -> bool {
        // see adjtimex(2)
        let mut buf: libc::timex = unsafe { std::mem::zeroed() };
        if unsafe { libc::adjtimex(&mut buf) } < 0 {
            false
        } else {
            // consider synced if within NTP_PHASE_LIMIT, relying on STA_UNSYNC isn't always reliable
            // see include/linux/timex.h and kernel/time/ntp.c for NTP_PHASE_LIMIT
            buf.maxerror < ((MAX_PHASE / NSEC_TO_USEC) << 5)
        }
    }

    /// show the current time on the system in µs
    #[dbus_interface(property(emits_changed_signal = "false"), name = "TimeUSec")]
    async fn time_usec(&self) -> fdo::Result<u64> {
        match nix::time::clock_gettime(nix::time::ClockId::CLOCK_REALTIME) {
            Ok(ts) => Ok(
                (
                    (ts.tv_sec() * SEC_TO_USEC) + (ts.tv_nsec() / NSEC_TO_USEC)
                ).try_into().unwrap_or_default()
            ),
            Err(e) => match e {
                Errno::ENOSYS => Err(fdo::Error::NotSupported("clock_gettime not supported on this system".into())),
                Errno::EINVAL => Err(fdo::Error::Failed("Unable to get current time".into())),
                _ => Err(fdo::Error::Failed(e.desc().into())),
            }
        }
    }

    /// show the current time in the RTC in µs
    #[dbus_interface(property(emits_changed_signal = "false"), name = "RTCTimeUSec")]
    async fn rtc_time_usec(&self) -> fdo::Result<u64> {
        // TODO: wtf why is this syscall failing
        match &self.rtc {
            Ok(fd) => {
                match rtc_read(fd.clone()).await {
                    Ok(mut tm) => Ok(unsafe { libc::timegm(&mut tm) * SEC_TO_USEC }.try_into().unwrap_or_default()),
                    Err(e) => Err(fdo::Error::Failed(e))
                }
            },
            Err(e) => Err(fdo::Error::Failed(format!("Couldn't open /dev/rtc: {}", e.desc()))),
        }
    }
}

#[tokio::main]
async fn main() -> zbus::Result<()> {
    let timedate = TimeDate {
        tz: std::env::var("TZ").ok(),
        rtc: rtc_open(),
    };
    let conn = Connection::system().await?;
    conn.object_server().at("/org/freedesktop/timedate1", timedate).await?;
    conn.request_name("org.freedesktop.timedate1").await?;

    loop {
        std::future::pending::<()>().await;
    }
}
