#![warn(clippy::doc_markdown, missing_debug_implementations)]
#![doc = include_str!("../README.md")]

/// The configuration module
pub mod config;

/// The actix module
pub mod actix;

use async_trait::async_trait;
use std::path::{Path, PathBuf};
pub use {actix::handler::ActixGitHttp, actix::router as actix_git_router};

#[derive(Clone, Debug)]
pub enum GitOperation {
    /// GET /info/refs?service=git-upload-pack
    InfoRefsUploadPack,
    /// GET /info/refs?service=git-receive-pack
    InfoRefsReceivePack,
    /// POST /git-upload-pack
    UploadPack,
    /// POST /git-receive-pack
    ReceivePack,
    /// GET text endpoints like HEAD or objects/info/*
    GetText,
    /// GET objects/info/packs
    ObjectsInfoPacks,
    /// GET objects/pack/*.pack or *.idx
    ObjectsPack,
    Other,
}

#[derive(Clone, Debug)]
pub struct AuthInput {
    pub authorization: Option<String>,
}

#[async_trait]
pub trait GitConfig {
    /// Rewrite the path
    async fn rewrite(&self, path: String) -> PathBuf;

    /// Authenticate current request using owned header data.
    /// Return Ok(()) to allow, Err(()) to reject.
    async fn authenticate(&self, _auth: AuthInput) -> Result<(), ()>;
    
    async fn is_public_repo(&self, _repo_path: &Path) -> bool;
    
    /// For public repos, this decides which operations can skip authentication.
    async fn allow_anonymous(&self, _op: GitOperation) -> bool;
}