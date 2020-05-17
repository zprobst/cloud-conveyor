use cloud_conveyor_core::{Action, Pipeline};

#[test]
fn test_pipeline_empty() {
    let mut pipeline = Pipeline::empty();
    assert_eq!(pipeline.pop_next_action(), None)
}

#[test]
fn test_pipeline_add_and_remove() {
    let action = Action::Build {
        sha: String::from("e56d336922eaab3be8c1244dbaa713e134a8eba50ddbd4f50fd2fe18d72595cd"),
        repo: String::from("hello"),
        artifact_bucket: String::from("hello"),
        artifact_folder: String::from("hello"),
    };

    let mut pipeline = Pipeline::empty();
    pipeline = pipeline.add_action(action.clone());
    pipeline = pipeline.add_action(action.clone());
    assert!(pipeline.pop_next_action().is_some());
    assert!(pipeline.pop_next_action().is_none());
}

#[test]
fn test_pipeline_cancel() {
    let action = Action::Build {
        sha: String::from("e56d336922eaab3be8c1244dbaa713e134a8eba50ddbd4f50fd2fe18d72595cd"),
        repo: String::from("hello"),
        artifact_bucket: String::from("hello"),
        artifact_folder: String::from("hello"),
    };

    let mut pipeline = Pipeline::empty();
    pipeline = pipeline.add_action(action.clone());
    pipeline = pipeline.add_action(action.clone());
    pipeline.cancel();
    assert!(pipeline.pop_next_action().is_none());
}
