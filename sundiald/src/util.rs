use std::{
    ffi::c_int,
    io::{self, BufRead},
    os::fd::RawFd,
    path::Path,
    ptr::addr_of,
};

use anyhow::Result;
use nix::{errno::Errno, fcntl::OFlag, ioctl_read, sys::stat::Mode};
use tokio::fs::File;

pub(crate) const SEC_TO_USEC: libc::c_long = 1_000_000;

// see linux/rtc.h
#[repr(C)]
pub struct RtcTime {
    pub sec: c_int,
    pub min: c_int,
    pub hour: c_int,
    pub mday: c_int,
    pub mon: c_int,
    pub year: c_int,
    pub wday: c_int,
    pub yday: c_int,
    pub isdst: c_int,
}

impl From<libc::tm> for RtcTime {
    fn from(value: libc::tm) -> Self {
        Self {
            sec: value.tm_sec,
            min: value.tm_min,
            hour: value.tm_hour,
            mday: value.tm_mday,
            mon: value.tm_mon,
            year: value.tm_year,
            wday: value.tm_wday,
            yday: value.tm_yday,
            isdst: value.tm_isdst,
        }
    }
}

impl From<RtcTime> for libc::tm {
    fn from(value: RtcTime) -> Self {
        let zone = 0;
        Self {
            tm_sec: value.sec,
            tm_min: value.min,
            tm_hour: value.hour,
            tm_mday: value.mday,
            tm_mon: value.mon,
            tm_year: value.year,
            tm_wday: value.wday,
            tm_yday: value.yday,
            tm_isdst: value.isdst,
            tm_gmtoff: 0,
            tm_zone: addr_of!(zone),
        }
    }
}

const RTC_MAGIC: u8 = b'p';
const RTC_RD_TIME_ID: u8 = 0x09;
const RTC_SET_TIME_ID: u8 = 0x0a;

ioctl_read!(rtc_read_time, RTC_MAGIC, RTC_RD_TIME_ID, RtcTime);
ioctl_read!(rtc_set_time, RTC_MAGIC, RTC_SET_TIME_ID, RtcTime);

fn rtc_open() -> nix::Result<RawFd> {
    nix::fcntl::open(
        "/dev/rtc",
        OFlag::O_RDONLY | OFlag::O_CLOEXEC,
        Mode::empty(),
    )
}

fn rtc_close(fd: RawFd) -> nix::Result<()> {
    nix::unistd::close(fd)
}

fn rtc_read(fd: RawFd) -> nix::Result<RtcTime> {
    let mut buf: RtcTime = unsafe { std::mem::zeroed() };
    match unsafe { rtc_read_time(fd, &mut buf) } {
        Ok(_) => Ok(buf),
        Err(e) => Err(e),
    }
}

fn rtc_write(fd: RawFd, tm: impl Into<RtcTime>) -> nix::Result<()> {
    let mut buf: RtcTime = tm.into();
    match unsafe { rtc_set_time(fd, &mut buf) } {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub(crate) fn get_hwclock() -> nix::Result<u64> {
    match rtc_open() {
        Ok(fd) => {
            let ret = match rtc_read(fd) {
                Ok(tm) => Ok((unsafe { libc::timegm(&mut tm.into()) } * SEC_TO_USEC)
                    .try_into()
                    .unwrap_or_default()),
                Err(e) => Err(e),
            };
            if let Err(e) = rtc_close(fd) {
                return Err(e);
            }
            ret
        }
        Err(e) => Err(e),
    }
}

pub(crate) fn set_hwclock(tm: impl Into<RtcTime>) -> nix::Result<()> {
    match rtc_open() {
        Ok(fd) => {
            let ret = match rtc_write(fd, tm) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            };
            if let Err(e) = rtc_close(fd) {
                return Err(e);
            }
            ret
        }
        Err(e) => Err(e),
    }
}

pub(crate) async fn read_lines<P: AsRef<Path>>(
    fp: P,
) -> io::Result<io::Lines<io::BufReader<std::fs::File>>> {
    let f = File::open(fp).await?.into_std().await;
    // TODO: use async
    Ok(io::BufReader::new(f).lines())
}
