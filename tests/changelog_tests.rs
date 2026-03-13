mod common;

use common::{OutputAssertions, TestRepo};

#[test]
fn test_changelog_explicit_from() {
    let repo = TestRepo::new();

    // Tag the initial commit
    repo.git(&["tag", "v1.0.0"]);

    // Add some commits after the tag
    repo.create_file("a.txt", "aaa");
    repo.commit("feat: add a");
    repo.create_file("b.txt", "bbb");
    repo.commit("fix: fix b (#42)");

    let output = repo.run_stax(&["changelog", "v1.0.0"]);
    output.assert_success();

    let stdout = TestRepo::stdout(&output);
    assert!(stdout.contains("feat: add a"));
    assert!(stdout.contains("fix: fix b"));
}

#[test]
fn test_changelog_from_last_tag() {
    let repo = TestRepo::new();

    // Tag the initial commit
    repo.git(&["tag", "v1.0.0"]);

    // Add commits after the tag
    repo.create_file("c.txt", "ccc");
    repo.commit("feat: add c");

    // Run changelog with no from arg — should auto-resolve to v1.0.0
    let output = repo.run_stax(&["changelog"]);
    output.assert_success();

    let stdout = TestRepo::stdout(&output);
    assert!(stdout.contains("feat: add c"));
    assert!(stdout.contains("resolved to latest tag"));
}

#[test]
fn test_changelog_tag_prefix() {
    let repo = TestRepo::new();

    // Create an ios release tag at initial commit
    repo.git(&["tag", "release/ios/v1.0.0"]);

    // Add a commit and create an android tag
    repo.create_file("android.txt", "droid");
    repo.commit("feat: android stuff");
    repo.git(&["tag", "release/android/v1.0.0"]);

    // Add more commits after both tags
    repo.create_file("shared.txt", "shared");
    repo.commit("feat: shared work");

    // Ask for changelog since last ios tag — should include android stuff + shared work
    let output = repo.run_stax(&["changelog", "--tag-prefix", "release/ios"]);
    output.assert_success();

    let stdout = TestRepo::stdout(&output);
    assert!(stdout.contains("feat: android stuff"));
    assert!(stdout.contains("feat: shared work"));

    // Ask for changelog since last android tag — should only include shared work
    let output = repo.run_stax(&["changelog", "--tag-prefix", "release/android"]);
    output.assert_success();

    let stdout = TestRepo::stdout(&output);
    assert!(!stdout.contains("feat: android stuff"));
    assert!(stdout.contains("feat: shared work"));
}

#[test]
fn test_changelog_no_tags_error() {
    let repo = TestRepo::new();

    // No tags exist — changelog with no args should fail
    let output = repo.run_stax(&["changelog"]);
    output.assert_failure();

    let stderr = TestRepo::stderr(&output);
    assert!(stderr.contains("No tags found"));
}

#[test]
fn test_changelog_prefix_no_match_error() {
    let repo = TestRepo::new();

    repo.git(&["tag", "release/android/v1.0.0"]);

    let output = repo.run_stax(&["changelog", "--tag-prefix", "release/ios"]);
    output.assert_failure();

    let stderr = TestRepo::stderr(&output);
    assert!(stderr.contains("release/ios"));
}

#[test]
fn test_changelog_json_includes_resolved_from() {
    let repo = TestRepo::new();

    repo.git(&["tag", "v2.0.0"]);
    repo.create_file("d.txt", "ddd");
    repo.commit("feat: add d");

    let output = repo.run_stax(&["changelog", "--json"]);
    output.assert_success();

    let stdout = TestRepo::stdout(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("invalid JSON");

    assert_eq!(json["from"], "v2.0.0");
    assert_eq!(json["resolved_from"], "v2.0.0");
    assert_eq!(json["commit_count"], 1);
}

#[test]
fn test_changelog_json_explicit_from_no_resolved() {
    let repo = TestRepo::new();

    repo.git(&["tag", "v1.0.0"]);
    repo.create_file("e.txt", "eee");
    repo.commit("feat: add e");

    let output = repo.run_stax(&["changelog", "v1.0.0", "--json"]);
    output.assert_success();

    let stdout = TestRepo::stdout(&output);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("invalid JSON");

    assert_eq!(json["from"], "v1.0.0");
    assert!(json["resolved_from"].is_null());
}
