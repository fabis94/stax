use crate::engine::BranchMetadata;
use crate::git::GitRepo;
use anyhow::Result;
use colored::Colorize;

/// Normalize parent metadata for a restack scope so branches with merged/missing
/// parents can be rebased onto trunk with provenance-aware boundaries.
pub(crate) fn normalize_scope_parents_for_restack(
    repo: &GitRepo,
    scope: &[String],
    quiet: bool,
) -> Result<usize> {
    let trunk = repo.trunk_branch()?;
    let mut normalized = 0usize;

    for branch in scope {
        let Some(meta) = BranchMetadata::read(repo.inner(), branch)? else {
            continue;
        };

        if meta.parent_branch_name == trunk {
            continue;
        }

        let parent_branch = meta.parent_branch_name.clone();
        let parent_tip = repo.branch_commit(&parent_branch).ok();
        let (should_reparent, reason) = if parent_tip.is_none() {
            (true, "missing")
        } else if repo
            .is_branch_merged_equivalent_to_trunk(&parent_branch)
            .unwrap_or(false)
        {
            (true, "merged")
        } else {
            (false, "")
        };

        if !should_reparent {
            continue;
        }

        // Only use the parent's current tip when it is actually in the
        // child's ancestry.  If the parent was rebased its tip may have moved
        // out of the child's commit graph (#120).
        let old_parent_boundary = parent_tip
            .filter(|tip| repo.is_ancestor(tip, branch).unwrap_or(false))
            .unwrap_or_else(|| meta.parent_branch_revision.clone());
        let updated_meta = BranchMetadata {
            parent_branch_name: trunk.clone(),
            parent_branch_revision: old_parent_boundary,
            ..meta
        };
        updated_meta.write(repo.inner(), branch)?;
        normalized += 1;

        if !quiet {
            let reason_text = if reason == "missing" {
                "parent missing"
            } else {
                "parent merged into trunk"
            };
            println!(
                "  {} normalized {} → {} ({})",
                "↪".cyan(),
                branch.cyan(),
                trunk.cyan(),
                reason_text.dimmed()
            );
        }
    }

    Ok(normalized)
}
