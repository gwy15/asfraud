use std::{future::Future, pin::Pin};

use crate::error::Result;
use actix_web::{
    dev::{HttpServiceFactory, Service, ServiceResponse},
    web::{self, Json},
    HttpResponse,
};

use crate::{db, models};

// async fn admin_html() -> HttpResponse {
//     #[cfg(not(debug_assertions))]
//     let html = include_str!("../../static/admin.html");
//     #[cfg(debug_assertions)]
//     let html = std::fs::read_to_string("static/admin.html").unwrap();
//     HttpResponse::Ok()
//         .content_type("text/html; charset=utf-8")
//         .body(html)
// }

pub fn service(token: &str) -> impl HttpServiceFactory {
    let token = token.to_string();
    web::scope("/admin").service(
        web::scope("/api")
            .wrap_fn(move |req, srv| {
                if req.headers().get("Authorization").map(|v| v.as_bytes())
                    != Some(token.as_bytes())
                {
                    let fut: Pin<Box<dyn Future<Output = actix_web::Result<ServiceResponse>>>> =
                        Box::pin(async {
                            let http = HttpResponse::Unauthorized().finish();
                            let rsp = ServiceResponse::new(req.into_parts().0, http);
                            Ok(rsp)
                        });
                    return fut;
                }
                let fut = srv.call(req);
                Box::pin(fut)
            })
            .route("/urls", web::get().to(list_urls))
            .route("/urls", web::post().to(create_url))
            .route("/urls/{id}", web::delete().to(delete_url))
            .route("/urls/{id}", web::put().to(update_url)),
    )
    // .default_service(web::route().to(admin_html))
}

pub async fn list_urls(p: web::Data<db::Pool>) -> Result<Json<Vec<models::Url>>> {
    Ok(Json(models::Url::list(&p).await?))
}

pub async fn create_url(data: web::Json<models::Url>, p: web::Data<db::Pool>) -> Result<Json<()>> {
    models::Url::insert(data.into_inner(), &p).await?;
    Ok(Json(()))
}

pub async fn delete_url(url_id: web::Path<i64>, p: web::Data<db::Pool>) -> Result<Json<()>> {
    models::Url::delete(url_id.into_inner(), &p).await?;
    Ok(Json(()))
}

pub async fn update_url(
    url_id: web::Path<i64>,
    data: web::Json<models::Url>,
    p: web::Data<db::Pool>,
) -> Result<Json<()>> {
    let mut url = data.into_inner();
    url.id = url_id.into_inner();

    url.update(&p).await?;
    Ok(Json(()))
}
