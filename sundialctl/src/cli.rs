use std::fmt::Display;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub(crate) struct Cli {
    /// Action to perform
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    /// Show current time and date settings (default action)
    Status,
    /// Show current time and date settings in a machine-readable format
    // Show,
    /// Set the system clock
    SetTime {
        /// The new time. Various formats are supported, see sundialctl(1) for more information
        time: String,
        /// Do not ask for a password (if necessary)
        #[arg(short, long)]
        noninteractive: bool,
    },
    /// Set the system timezone
    SetTimezone {
        /// The new timezone
        zone: String,
        /// Do not ask for a password (if necessary)
        #[arg(short, long)]
        noninteractive: bool,
    },
    /// List all available timezones
    ListTimezones,
    /// Set the real-time clock to use UTC or local time
    SetRTC {
        /// New RTC mode
        mode: RtcMode,
        /// After setting the RTC mode, sync the time from the system clock to the RTC or vice versa.
        #[arg(short, long, required = false, default_value_t = RtcSyncFrom::Sys, value_name = "CLOCK")]
        sync_from: RtcSyncFrom,
        /// Do not ask for a password (if necessary)
        #[arg(short, long)]
        noninteractive: bool,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub(crate) enum RtcMode {
    Utc,
    Local,
}

impl From<RtcMode> for bool {
    fn from(value: RtcMode) -> Self {
        match value {
            RtcMode::Local => true,
            RtcMode::Utc => false,
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub(crate) enum RtcSyncFrom {
    Sys,
    Rtc,
}

impl Default for RtcSyncFrom {
    fn default() -> Self {
        Self::Sys
    }
}

impl Display for RtcSyncFrom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rtc => "rtc".fmt(f),
            Self::Sys => "sys".fmt(f),
        }
    }
}

impl From<RtcSyncFrom> for bool {
    fn from(value: RtcSyncFrom) -> Self {
        match value {
            RtcSyncFrom::Rtc => true,
            RtcSyncFrom::Sys => false,
        }
    }
}
