use anyhow::Result;
use clap::Parser;
use zbus::Connection;

use crate::{
    cli::{Cli, Commands},
    dbus::timedate1Proxy,
};

mod cli;
mod dbus;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    dbg!(&cli);

    let conn = Connection::system().await?;
    let proxy = timedate1Proxy::new(&conn).await?;

    match cli.command.unwrap_or(Commands::Status) {
        Commands::Status => proxy.status_cmd(true).await,
        // Commands::Show => proxy.status_cmd(false).await,
        Commands::SetTime {
            time,
            noninteractive,
        } => proxy.set_time_cmd(time, !noninteractive).await,
        Commands::ListTimezones => proxy.list_tz_cmd().await,
        Commands::SetTimezone {
            zone,
            noninteractive,
        } => proxy.set_tz_cmd(zone, !noninteractive).await,
        Commands::SetRTC {
            mode,
            sync_from,
            noninteractive,
        } => proxy.set_rtc_cmd(mode, sync_from, !noninteractive).await,
    }
}
