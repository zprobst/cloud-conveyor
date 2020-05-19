#[cfg(test)]
use cloud_conveyor_core::pipelining::{Build, Pipeline};

#[test]
fn pipeline_allows_one_build() {
    let mut pipeline = Pipeline::default();
    let sha = "some_sha_here";
    let repo = "some_repo_here";
    let buildOne = Build {
        sha: sha.to_string(),
        repo: repo.to_string(),
    };
    let buildTwo = Build {
        sha: sha.to_string(),
        repo: repo.to_string(),
    };
    pipeline = pipeline
        .add_action(Box::new(buildOne))
        .add_action(Box::new(buildTwo));
    assert!(pipeline.pop_next_action().is_some());
    assert!(pipeline.pop_next_action().is_none());
}
