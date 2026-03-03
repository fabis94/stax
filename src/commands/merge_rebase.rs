use crate::engine::BranchMetadata;
use crate::git::{GitRepo, RebaseResult};
use anyhow::Result;

pub(crate) fn fetch_remote_for_descendant_rebase(
    repo: &GitRepo,
    remote_name: &str,
) -> Result<bool> {
    repo.fetch_remote(remote_name)
}

pub(crate) fn rebase_descendant_onto_remote_trunk_with_provenance(
    repo: &GitRepo,
    branch: &str,
    trunk: &str,
    remote_name: &str,
) -> Result<RebaseResult> {
    let remote_trunk_ref = format!("{}/{}", remote_name, trunk);
    let fallback_upstream = BranchMetadata::read(repo.inner(), branch)?
        .map(|meta| meta.parent_branch_revision)
        .unwrap_or_default();

    let result = repo.rebase_branch_onto_with_provenance(
        branch,
        &remote_trunk_ref,
        &fallback_upstream,
        false,
    )?;

    if result == RebaseResult::Success {
        if let Some(meta) = BranchMetadata::read(repo.inner(), branch)? {
            let trunk_commit = repo
                .resolve_ref(&remote_trunk_ref)
                .unwrap_or_else(|_| repo.branch_commit(trunk).unwrap_or_default());
            let updated_meta = BranchMetadata {
                parent_branch_name: trunk.to_string(),
                parent_branch_revision: trunk_commit,
                ..meta
            };
            updated_meta.write(repo.inner(), branch)?;
        }
    }

    Ok(result)
}
