use crate::actix::ensure_auth;
use crate::{GitConfig, GitOperation};
use actix_files::NamedFile;
use actix_web::cookie::time;
use actix_web::cookie::time::format_description;
use actix_web::http::header;
use actix_web::http::header::HeaderValue;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use std::collections::HashMap;

pub async fn objects_info_packs(
    request: HttpRequest,
    service: web::Data<impl GitConfig>,
) -> impl Responder {
    let uri = request.uri();
    let path = uri.path().to_string();
    let repo_path = service.rewrite(path.clone()).await;

    if let Err(resp) = ensure_auth(&request, &service, &repo_path, GitOperation::ObjectsInfoPacks).await {
        return resp;
    }

    let path = "objects/info/packs".to_string();
    let mut map = HashMap::new();
    let time = time::OffsetDateTime::now_utc();
    let expires = time::OffsetDateTime::now_utc() + time::Duration::days(1);
    map.insert(
        "Date".to_string(),
        time.format(&format_description::parse("%a, %d %b %Y %H:%M:%S GMT").unwrap())
            .unwrap(),
    );
    map.insert(
        "Expires".to_string(),
        expires
            .format(&format_description::parse("%a, %d %b %Y %H:%M:%S GMT").unwrap())
            .unwrap(),
    );
    map.insert(
        "Cache-Control".to_string(),
        "public, max-age=86400".to_string(),
    );
    let req_file = repo_path.join(path);
    if !req_file.exists() {
        return HttpResponse::NotFound().body("File not found");
    }
    match NamedFile::open(req_file) {
        Ok(mut named_file) => {
            named_file = named_file.use_last_modified(true);
            let mut response = named_file.into_response(&request);
            for (k, v) in map.iter() {
                response.headers_mut().insert(
                    k.to_string().parse().unwrap(),
                    HeaderValue::from_str(v).unwrap(),
                );
            }

            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str("application/x-git-loose-object").unwrap(),
            );
            response
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to open file"),
    }
}