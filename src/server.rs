use actix_web::{web, App, HttpServer};
use clap::Parser;
use git_http_backend::actix::handler::ActixGitHttp;
use git_http_backend::actix_git_router;
use git_http_backend::config::GitHttpConfig;
use git_http_backend::{AuthInput, GitConfig, GitOperation};
use std::io;
use std::path::PathBuf;
use tracing;
use std::fs;
use http_auth_basic::Credentials;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct ActixServerArgs {
    #[arg(short, long, default_value = "e:")]
    pub root: String,
    #[arg(short, long, default_value = "80")]
    pub port: u16,
    #[arg(short, long, default_value = "0.0.0.0")]
    pub addr: String,
}

#[derive(Clone, Debug)]
struct WithAuth {
    inner: ActixGitHttp,
}

#[async_trait::async_trait]
impl GitConfig for WithAuth {
    async fn rewrite(&self, path: String) -> PathBuf {
        self.inner.rewrite(path).await
    }

    async fn authenticate(&self, auth: AuthInput) -> Result<(), ()> {
        let expected = Credentials::new("username", "password");
        if let Some(h) = auth.authorization {
            let credentials = Credentials::from_header(h).unwrap();
            tracing::info!(credentials.user_id, credentials.password);
            if credentials == expected {
                return Ok(());
            }
        }
        Err(())
    }

    async fn is_public_repo(&self, repo_path: &std::path::Path) -> bool {
        true
    }

    async fn allow_anonymous(&self, op: GitOperation) -> bool {
        self.inner.allow_anonymous(op).await
    }
}

#[tokio::main]
pub async fn main() -> io::Result<()> {
    tracing_subscriber::fmt().init();
    let args = ActixServerArgs::parse();
    let root = PathBuf::from(args.root);

    if !root.exists() {
        tracing::warn!("root path not exists");
        fs::create_dir_all(root.clone())?;
    }

    let base = ActixGitHttp {
        config: GitHttpConfig {
            root,
            port: args.port,
            addr: args.addr,
        },
    };

    let auth = WithAuth { inner: base };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(auth.clone()))
            .wrap(actix_web::middleware::Logger::default())
            .configure(|x| actix_git_router::<WithAuth>(x))
    })
        .bind("0.0.0.0:80")?
        .run()
        .await
}