use crate::actix::ensure_auth;
use crate::{GitConfig, GitOperation};
use actix_web::http::header::HeaderValue;
use actix_web::http::StatusCode;
use actix_web::{web, HttpRequest, HttpResponseBuilder, Responder};
use std::process::Command;
use tracing::info;

pub async fn info_refs(request: HttpRequest, service: web::Data<impl GitConfig>) -> impl Responder {
    let uri = request.uri();
    let path = uri.path().to_string().replace("/info/refs", "");
    let path = service.rewrite(path).await;

    let query = uri.query().unwrap_or("");
    let svc_parts = query.split('=').map(|x| x.to_string()).collect::<Vec<_>>();
    let svc_raw = svc_parts.get(1).cloned().unwrap_or_else(|| "".to_string());

    if !(svc_raw != "git-upload-pack" || svc_raw != "git-receive-pack") {
        return Err(actix_web::error::ErrorUnsupportedMediaType(
            "service not git-upload-pack",
        ));
    }
    let service_name = svc_raw.replace("git-", "");
    
    let op = match service_name.as_str() {
        "upload-pack" => GitOperation::InfoRefsUploadPack,
        "receive-pack" => GitOperation::InfoRefsReceivePack,
        _ => GitOperation::Other,
    };
    if let Err(resp) = ensure_auth(&request, &service, &path, op).await {
        return Ok(resp);
    }

    let version = request
        .headers()
        .get("Git-Protocol")
        .unwrap_or(&HeaderValue::from_str("").unwrap())
        .to_str()
        .map(|s| s.to_string())
        .unwrap_or("".to_string());

    let mut cmd = Command::new("git");
    cmd.arg(service_name.clone());
    cmd.arg("--stateless-rpc");
    cmd.arg("--advertise-refs");
    cmd.arg(".");
    cmd.current_dir(path);
    if !version.is_empty() {
        cmd.env("GIT_PROTOCOL", version.clone());
    }

    let output = match cmd.output() {
        Ok(output) => {
            info!("Command status: {:?}", output.status);
            output
        }
        Err(e) => {
            eprintln!("Error running command: {}", e);
            return Err(actix_web::error::ErrorInternalServerError(
                "Error running command",
            ));
        }
    };
    let mut resp = HttpResponseBuilder::new(StatusCode::OK);
    resp.append_header((
        "Content-Type",
        format!("application/x-git-{}-advertisement", service_name),
    ));
    resp.append_header(("Pragma", "no-cache"));
    resp.append_header(("Cache-Control", "no-cache, max-age=0, must-revalidate"));
    resp.append_header(("Expires", "Fri, 01 Jan 1980 00:00:00 GMT"));

    let mut body = String::new();

    match service_name.as_str() {
        "upload-pack" => {
            body.push_str(&"001e# service=git-upload-pack\n".to_string());
            body.push_str("0000");
        }
        "receive-pack" => {
            body.push_str(&"001f# service=git-receive-pack\n".to_string());
            body.push_str("0000");
        }
        _ => {}
    }
    body.push_str(&String::from_utf8(output.stdout).unwrap());
    Ok(resp.body(body.as_bytes().to_vec()))
}