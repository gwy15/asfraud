use std::borrow::Cow;

use actix_web::{web, App, HttpRequest, HttpResponse, HttpResponseBuilder, HttpServer};
use anyhow::{Context, Result};
use clap::Parser;
use http::StatusCode;
use serde_json::json;
use tracing::*;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(long, default_value = "8080")]
    port: u16,
}

async fn route(req: HttpRequest) -> HttpResponse {
    match handle_request(req).await {
        Ok(response) => response,
        Err(e) => HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR).json(json!({
            "errmsg": e.to_string(),
            "detail": format!("{:?}", e),
        })),
    }
}

#[instrument(skip(req))]
async fn handle_request(req: HttpRequest) -> Result<HttpResponse> {
    let path = req.path();
    let ua = req
        .headers()
        .get(http::header::USER_AGENT)
        .context("missing UA")?
        .to_str()?;
    debug!(%path, %ua, "request");

    let response = if is_lark_bot_ua(ua) {
        info!("lark bot detected, redirecting to html");
        let html = html(path).await?.into_owned();
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html)
    } else {
        info!("non bot, redirecting to bilibili");
        HttpResponse::TemporaryRedirect()
            .append_header((
                http::header::LOCATION,
                "https://www.bilibili.com/video/BV1MX4y1N75X",
            ))
            .finish()
    };
    Ok(response)
}

fn is_lark_bot_ua(ua: &str) -> bool {
    // 飞书的爬虫包含这个，就算误差一点正常用户也无所谓了
    ua.contains("Chrome/91.0.4450.0")
}

async fn html(path: &str) -> Result<Cow<'static, str>> {
    lazy_static::lazy_static! {
        static ref FILENAME: regex::Regex = regex::Regex::new(r#"(?P<name>\w+)(\.html)?"#).unwrap();
        static ref ROOT: std::path::PathBuf = "./static".into();
    }
    const EMBEDDED_HTML: &str = include_str!("../static/index.html");

    let filename = match FILENAME.captures(path) {
        Some(cap) => cap.name("name").unwrap().as_str(),
        None => {
            debug!(%path, "invalid path, fallback to index.html");
            "index"
        }
    };

    let mut path = ROOT.clone();
    path.push(&format!("{filename}.html"));

    let s = tokio::fs::read_to_string(&path)
        .await
        .map(Cow::from)
        .unwrap_or_else(|e| {
            debug!(?e, "file not found, fallback to embedded index.html");
            Cow::from(EMBEDDED_HTML)
        });
    Ok(s)
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();

    HttpServer::new(|| App::new().default_service(web::route().to(route)))
        .bind(("0.0.0.0", args.port))?
        .run()
        .await?;
    Ok(())
}
