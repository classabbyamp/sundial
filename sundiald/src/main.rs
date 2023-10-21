use anyhow::{Context, Result};
use zbus::Connection;
use zbus_polkit::policykit1::{AuthorityProxy, Subject};

use crate::dbus::TimeDate;

#[macro_use]
extern crate log;

mod dbus;
mod util;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let conn = Connection::system()
        .await
        .context("Failed to connect to system D-Bus")?;
    debug!("Connected to system D-Bus");
    let timedate = TimeDate {
        auth: AuthorityProxy::new(&conn)
            .await
            .context("Failed to connect to PolicyKit")?,
        subject: Subject::new_for_owner(std::process::id(), None, None)
            .context("Failed to get PolicyKit subject")?,
    };
    conn.object_server()
        .at("/org/freedesktop/timedate1", timedate)
        .await
        .context("Failed to register D-Bus interface")?;
    conn.request_name("org.freedesktop.timedate1")
        .await
        .context("Failed to register D-Bus name")?;
    info!("Listening on system D-Bus");

    loop {
        std::future::pending::<()>().await;
    }
}
