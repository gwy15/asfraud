use crate::{
    db,
    error::{Context, Result},
    models,
};
use actix_web::{web, HttpRequest, HttpResponse};

pub mod admin;

fn is_lark_bot_ua(ua: &str) -> bool {
    // 飞书的爬虫包含这个，就算误差一点正常用户也无所谓了
    ua.contains("Chrome/91.0.4450.0")
}

fn redirect(url: &str) -> HttpResponse {
    HttpResponse::TemporaryRedirect()
        .append_header((http::header::LOCATION, url))
        .finish()
}

pub async fn empty() -> &'static str {
    ""
}

pub async fn handle_request(req: HttpRequest, p: web::Data<db::Pool>) -> Result<HttpResponse> {
    let path = req.path();
    let ua = req
        .headers()
        .get(http::header::USER_AGENT)
        .context("missing UA")?
        .to_str()
        .context("Invalid UA")?;
    debug!(%path, %ua, "request");

    let url = models::Url::from_path(path, &p).await?;
    let Some(url) = url else {
            return Ok(redirect("https://www.bilibili.com/video/BV1MX4y1N75X"));
        };

    let response = if is_lark_bot_ua(ua) {
        debug!("lark bot detected, redirecting to html");
        let html = url.html();
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html)
    } else {
        info!("non bot, redirecting to bilibili");
        let url_id = url.id;
        tokio::spawn(async move {
            models::Url::incr(url_id, &p).await.ok();
        });
        redirect(&url.redirect)
    };
    Ok(response)
}
