use crate::actix::get_text_file::get_text_file;
use crate::actix::git_receive_pack::git_receive_pack;
use crate::actix::git_upload_pack::git_upload_pack;
use crate::actix::objects_info_packs::objects_info_packs;
use crate::actix::objects_pack::objects_pack;
use crate::actix::refs::info_refs;
use crate::GitConfig;
use actix_web::web::Data;
use actix_web::{web, HttpRequest};

/// Actix-web DataServer
pub mod handler;

/// get text file handler
pub mod get_text_file;
/// receive-pack handler
pub mod git_receive_pack;
/// upload-pack handler
pub mod git_upload_pack;
/// objects info packs handler
pub mod objects_info_packs;
/// refs handler
pub mod refs;

/// objects pack handler
pub mod objects_pack;

use actix_web::HttpResponse;
use std::path::PathBuf;
use crate::{AuthInput, GitOperation};

pub(crate) async fn ensure_auth<T: GitConfig>(
    req: &HttpRequest,
    service: &Data<T>,
    repo_path: &PathBuf,
    op: GitOperation,
) -> Result<(), HttpResponse> {
    let is_public = service.is_public_repo(repo_path).await;
    let anon_ok = service.allow_anonymous(op).await;

    if is_public && anon_ok {
        return Ok(());
    }

    // Build owned auth input so the future remains Send.
    let authorization = req
        .headers()
        .get(actix_web::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_owned());

    match service.authenticate(AuthInput { authorization }).await {
        Ok(()) => Ok(()),
        Err(()) => Err(
            HttpResponse::Unauthorized()
                .append_header(("WWW-Authenticate", "Basic realm=\"git\""))
                .finish(),
        ),
    }
}

/// Actix-web Router Export
pub fn router<T>(cfg: &mut web::ServiceConfig)
where
    T: GitConfig + 'static,
{
    cfg.service(
        web::scope("/{namespace}/{repo}")
            .route(
                "/git-upload-pack",
                web::to::<_, (HttpRequest, web::Payload, Data<T>)>(git_upload_pack),
            )
            .route(
                "/git-receive-pack",
                web::to::<_, (HttpRequest, web::Payload, Data<T>)>(git_receive_pack),
            )
            .route("info/refs", web::to::<_, (HttpRequest, Data<T>)>(info_refs))
            .route("HEAD", web::to::<_, (HttpRequest, Data<T>)>(get_text_file))
            .route(
                "objects/info/alternates",
                web::to::<_, (HttpRequest, Data<T>)>(get_text_file),
            )
            .route(
                "objects/info/http-alternates",
                web::to::<_, (HttpRequest, Data<T>)>(get_text_file),
            )
            .route(
                "objects/info/packs",
                web::to::<_, (HttpRequest, Data<T>)>(objects_info_packs),
            )
            .route(
                "objects/info/{rest:.*}",
                web::to::<_, (HttpRequest, Data<T>)>(get_text_file),
            )
            .route(
                "objects/pack/{pack}",
                web::to::<_, (HttpRequest, Data<T>)>(objects_pack),
            ),
    );
}