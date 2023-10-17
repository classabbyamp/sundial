use std::{path::Path, os::fd::RawFd};

use enumflags2::BitFlag;
use libc::BLKSSZGET;
use nix::errno::Errno;
use tokio::fs::canonicalize;
use zbus::{dbus_interface, Connection, fdo};
use zbus_polkit::policykit1::{AuthorityProxy, Subject, CheckAuthorizationFlags};

use util::{rtc_open, rtc_close, rtc_read, read_lines};

mod util;

const SEC_TO_USEC: libc::c_long = 1_000_000;
const NSEC_TO_USEC: libc::c_long = 1_000;
const MAX_PHASE: libc::c_long = 500_000_000;

struct TimeDate {
    tz: Option<String>,
    auth: AuthorityProxy<'static>,
    subject: Subject,
}

#[dbus_interface(name = "org.freedesktop.timedate1")]
impl TimeDate {
    /// change the system clock
    async fn set_time(&self, usec_utc: i64, relative: bool, interactive: bool) -> fdo::Result<()> {
        // TODO
        // https://github.com/systemd/systemd/blob/main/src/timedate/timedated.c#L820

        // get a starting timestamp now

        // TODO: if ntp enabled, fail

        if relative {
            if usec_utc == 0 {
                return Ok(());
            }

            // get now()
            // now + usec_utc
            // ensure no overflow/underflow

        } else {
            if usec_utc <= 0 {
                return Err(fdo::Error::InvalidArgs("Invalid absolute time".into()));
            }
        }

        // polkit verify
        self.check_auth("org.freedesktop.timedate1.set-time", interactive).await?;
        // adjust for time spent: add now - starting timestamp
        // set system clock
        // sync from sysclock to rtc

        Ok(())
    }

    /// set the system timezone
    async fn set_timezone(&self, timezone: String, interactive: bool) -> fdo::Result<()> {
        // TODO
        // https://github.com/systemd/systemd/blob/main/src/timedate/timedated.c#L657
        // check if valid tz (return if not)
        // check if is current tz (return if true)
        // check polkit auth
        self.check_auth("org.freedesktop.timedate1.set-timezone", interactive).await?;
        // write new localtime symlink
        // tzset
        // tell kernel new tz
        // if local rtc, sync rtc from sysclock
        Ok(())
    }

    /// control whether the RTC is in local time or UTC
    #[dbus_interface(name = "SetLocalRTC")]
    async fn set_local_rtc(&self, local_rtc: bool, fix_system: bool, interactive: bool) -> fdo::Result<()> {
        // TODO
        // https://github.com/systemd/systemd/blob/main/src/timedate/timedated.c#L734
        // if local_rtc matches current state and not fix_system, return
        let curr = self.local_rtc().await?;
        if local_rtc == curr && ! fix_system {
            return Ok(());
        }

        // check polkit for auth
        self.check_auth("org.freedesktop.timedate1.set-local-rtc", interactive).await?;
        // if local_rtc doesn't match, change it
        if local_rtc != curr {
            // change value
            todo!()
        }
        // tell kernel the tz
        // sync clocks
        // if fix_system, sync system clock from rtc
        // else sync rtc from system clock
        // emit prop localrtc changed
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
        match read_lines("/usr/share/zoneinfo/zone.tab").await {
            Ok(lines) => {
                // grab the third word of each non-comment line
                Ok(lines.filter(|ln| ln.is_ok() && ! ln.as_ref().unwrap().starts_with('#'))
                        .filter_map(|ln| ln.unwrap().split_whitespace().nth(2).map(str::to_string))
                        .collect())
            }
            Err(e) => Err(fdo::Error::Failed(format!("Couldn't get timezone list: {}", e))),
        }
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
        match read_lines("/etc/adjtime").await {
            Err(e) => Err(fdo::Error::Failed(format!("Couldn't get RTC status: {}", e))),
            Ok(mut lines) => {
                if let Some(Ok(ln)) = lines.nth(2) {
                    Ok(ln == "LOCAL")
                } else {
                    Ok(false)  // assume UTC otherwise
                }
            }
        }
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
            // see include/linux/timex.h and kernel/time/ntp.c (NTP_PHASE_LIMIT)
            buf.maxerror < ((MAX_PHASE / NSEC_TO_USEC) << 5)
        }
    }

    /// show the current time on the system in µs
    #[dbus_interface(property(emits_changed_signal = "false"), name = "TimeUSec")]
    async fn time_usec(&self) -> fdo::Result<u64> {
        match nix::time::clock_gettime(nix::time::ClockId::CLOCK_REALTIME) {
            Ok(ts) => Ok((
                (ts.tv_sec() * SEC_TO_USEC) + (ts.tv_nsec() / NSEC_TO_USEC)
            ).try_into().unwrap_or_default()),
            Err(e) => match e {
                Errno::ENOSYS => Err(fdo::Error::NotSupported("clock_gettime not supported on this system".into())),
                Errno::EINVAL => Err(fdo::Error::Failed("Unable to get current time".into())),
                _ => Err(fdo::Error::Failed(format!("Unable to get current time: {}", e.desc()))),
            }
        }
    }

    /// show the current time in the RTC in µs
    #[dbus_interface(property(emits_changed_signal = "false"), name = "RTCTimeUSec")]
    async fn rtc_time_usec(&self) -> fdo::Result<u64> {
        match rtc_open() {
            Ok(fd) => {
                let ret = match rtc_read(fd.clone()).await {
                    Ok(tm) => Ok(unsafe { libc::timegm(&mut tm.into()) * SEC_TO_USEC }.try_into().unwrap_or_default()),
                    Err(e) => Err(fdo::Error::Failed(e))
                };
                if let Err(e) = rtc_close(fd) {
                    return Err(fdo::Error::Failed(e.desc().into()));
                }
                ret
            },
            Err(e) => Err(fdo::Error::Failed(format!("Couldn't open /dev/rtc: {}", e.desc()))),
        }
    }

    async fn check_auth(&self, action: &str, interactive: bool) -> zbus::fdo::Result<()> {
        let auth_res = self.auth.check_authorization(
            &self.subject,
            action,
            &std::collections::HashMap::new(),
            if interactive { CheckAuthorizationFlags::AllowUserInteraction.into() }
                else { CheckAuthorizationFlags::empty() },
            ""
        ).await?;

        if auth_res.is_authorized {
            Ok(())
        } else if auth_res.is_challenge {
            Err(zbus::fdo::Error::AuthFailed("Interactive authentication required".into()))
        } else {
            match caps::has_cap(None, caps::CapSet::Effective, caps::Capability::CAP_SYS_TIME) {
                Ok(true) => Ok(()),
                Ok(false) | Err(_) => Err(zbus::fdo::Error::AuthFailed("Does not have CAP_SYS_TIME".into()))
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::system().await?;
    let timedate = TimeDate {
        tz: std::env::var("TZ").ok(),
        auth: AuthorityProxy::new(&conn).await?,
        subject: Subject::new_for_owner(std::process::id(), None, None)?,
    };
    conn.object_server().at("/org/freedesktop/timedate1", timedate).await?;
    conn.request_name("org.freedesktop.timedate1").await?;

    loop {
        std::future::pending::<()>().await;
    }
}
