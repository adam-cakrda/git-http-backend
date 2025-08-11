use crate::config::GitHttpConfig;
use crate::{AuthInput, GitConfig, GitOperation};
use async_trait::async_trait;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone,Debug)]
pub struct ActixGitHttp {
    pub config: GitHttpConfig,
}

#[async_trait]
impl GitConfig for ActixGitHttp {
    async fn rewrite(&self, path: String) -> PathBuf {
        PathBuf::from(&self.config.root).join(path)
    }

    async fn authenticate(&self, _auth: AuthInput) -> Result<(), ()> {
        // Default: reject when authentication is required.
        Err(())
    }

    async fn is_public_repo(&self, repo_path: &Path) -> bool {
        if repo_path.join("git-daemon-export-ok").exists() {
            return true;
        }
        let cfg = repo_path.join("config");
        if let Ok(content) = fs::read_to_string(cfg) {
            if content
                .lines()
                .any(|l| l.trim().eq_ignore_ascii_case("http.allowAnonymous = true"))
            {
                return true;
            }
        }
        false
    }

    async fn allow_anonymous(&self, op: GitOperation) -> bool {
        match op {
            GitOperation::InfoRefsUploadPack
            | GitOperation::UploadPack
            | GitOperation::GetText
            | GitOperation::ObjectsInfoPacks
            | GitOperation::ObjectsPack => true,
            GitOperation::InfoRefsReceivePack | GitOperation::ReceivePack => false,
            GitOperation::Other => false,
        }
    }
}