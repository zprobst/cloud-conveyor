#[cfg(test)]
use cloud_conveyor_core::pipelining::{Build, Pipeline};

#[test]
fn pipeline_allows_one_build() {
    let mut pipeline = Pipeline::default();
    let sha = "some_sha_here";
    let repo = "some_repo_here";
    let build_one = Build::new(sha.to_string(), repo.to_string());
    let build_two = Build::new(sha.to_string(), repo.to_string());
    pipeline = pipeline
        .add_action(Box::new(build_one))
        .add_action(Box::new(build_two));
    assert!(pipeline.pop_next_action().is_some());
    assert!(pipeline.pop_next_action().is_none());
}
