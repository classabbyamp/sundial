use zbus::Connection;
use zbus_polkit::policykit1::{AuthorityProxy, Subject};

use crate::dbus::TimeDate;

mod dbus;
mod util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::system().await?;
    let timedate = TimeDate {
        tz: std::env::var("TZ").ok(),
        auth: AuthorityProxy::new(&conn).await?,
        subject: Subject::new_for_owner(std::process::id(), None, None)?,
    };
    conn.object_server()
        .at("/org/freedesktop/timedate1", timedate)
        .await?;
    conn.request_name("org.freedesktop.timedate1").await?;

    loop {
        std::future::pending::<()>().await;
    }
}
