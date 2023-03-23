use std::{os::fd::RawFd, path::Path, io::{self, BufRead}};

use nix::{ioctl_read, fcntl::OFlag, sys::stat::Mode, errno::Errno};
use tokio::fs::File;

const RTC_MAGIC: u8 = b'p';
const RTC_RD_TIME_ID: u8 = 0x09;

ioctl_read!(rtc_rd_time, RTC_MAGIC, RTC_RD_TIME_ID, libc::tm);

pub(crate) fn rtc_open() -> Result<RawFd, Errno> {
    nix::fcntl::open("/dev/rtc", OFlag::O_RDONLY | OFlag::O_CLOEXEC, Mode::empty())
}

pub(crate) async fn rtc_read(fd: RawFd) -> Result<libc::tm, String> {
    let mut buf: libc::tm = unsafe { std::mem::zeroed() };
    match unsafe { rtc_rd_time(fd, &mut buf) } {
        Ok(_) => Ok(buf),
        Err(e) => Err(e.desc().into()),
    }
}

pub(crate) async fn read_lines<P: AsRef<Path>>(fp: P) -> io::Result<io::Lines<io::BufReader<std::fs::File>>> {
    let f = File::open(fp).await?.into_std().await;
    // TODO: use async
    Ok(io::BufReader::new(f).lines())
}
