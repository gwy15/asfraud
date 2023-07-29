#[macro_use]
extern crate tracing;

mod db;
mod error;
mod models;
mod routes;

#[cfg(unix)]
use std::path::PathBuf;

use actix_web::{web, App, HttpServer};
use anyhow::{Context, Result};
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[cfg(unix)]
    #[clap(long = "socket", help = "Path to the unix socket")]
    socket: Option<PathBuf>,
    #[cfg(unix)]
    #[clap(
        long,
        default_value = "8080",
        help = "Port to listen on, ignored if --unix-socket is set"
    )]
    port: u16,

    #[cfg(not(unix))]
    #[clap(long, default_value = "8080", help = "Port to listen on")]
    port: u16,

    #[clap(long, default_value = "sqlite://data.db", help = "Database url")]
    db_url: String,

    #[clap(long, help = "admin token")]
    admin_token: String,
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();
    let db = crate::db::new(&args.db_url)
        .await
        .context("Open sqlite db failed")?;

    sqlx::migrate!().run(&db).await?;
    let db = web::Data::new(db);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(db.clone())
            .route("/favicon.ico", web::route().to(routes::empty))
            .service(routes::admin::service(&args.admin_token))
            .default_service(web::route().to(routes::handle_request))
    });

    #[cfg(unix)]
    if let Some(socket) = args.socket {
        server
            .bind_uds(socket)
            .context("Bind unix socket failed")?
            .run()
            .await?;
        return Ok(());
    }

    server.bind(("0.0.0.0", args.port))?.run().await?;
    Ok(())
}
