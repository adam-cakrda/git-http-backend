use crate::actix::ensure_auth;
use crate::{GitConfig, GitOperation};
use actix_files::NamedFile;
use actix_web::http::header;
use actix_web::http::header::HeaderValue;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use std::collections::HashMap;

fn repo_prefix_from_uri_path(path: &str) -> String {
    if let Some(i) = path.find(".git") {
        path[..i + 4].to_string()
    } else {
        path.to_string()
    }
}

fn op_from_path(path: &str) -> GitOperation {
    if path.contains("objects/pack/") {
        GitOperation::ObjectsPack
    } else if path.contains("objects/info/packs") {
        GitOperation::ObjectsInfoPacks
    } else {
        GitOperation::GetText
    }
}

pub async fn get_text_file(
    request: HttpRequest,
    service: web::Data<impl GitConfig>,
) -> impl Responder {
    let uri = request.uri();
    let path = uri.path().to_string();

    // Determine repo root for auth
    let repo_prefix = repo_prefix_from_uri_path(&path);
    let repo_path = service.rewrite(repo_prefix).await;
    let op = op_from_path(&path);

    if let Err(resp) = ensure_auth(&request, &service, &repo_path, op).await {
        return resp;
    }

    let path = service.rewrite(path).await;
    let mut resp = HashMap::new();
    resp.insert("Pragma".to_string(), "no-cache".to_string());
    resp.insert(
        "Cache-Control".to_string(),
        "no-cache, max-age=0, must-revalidate".to_string(),
    );
    resp.insert(
        "Expires".to_string(),
        "Fri, 01 Jan 1980 00:00:00 GMT".to_string(),
    );
    if !path.exists() {
        return HttpResponse::NotFound().body("File not found");
    }
    match NamedFile::open(path) {
        Ok(mut named_file) => {
            named_file = named_file.use_last_modified(true);
            let mut response = named_file.into_response(&request);
            for (k, v) in resp.iter() {
                response.headers_mut().insert(
                    k.to_string().parse().unwrap(),
                    HeaderValue::from_str(v).unwrap(),
                );
            }

            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str("text/plain").unwrap(),
            );
            response
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to open file"),
    }
}