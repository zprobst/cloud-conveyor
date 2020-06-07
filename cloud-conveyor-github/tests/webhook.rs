use cloud_conveyor_core::webhook::{InterpretWebhooks, VcsEvent, WebhookRequest};
use cloud_conveyor_github::Github;

use std::collections::HashMap;
use std::fs;

fn generate_request_from_file(file_name: &str) -> WebhookRequest {
    let mut headers = HashMap::new();
    headers.insert("X-Hub-Signature".to_owned(), "hmac=test".to_owned());
    WebhookRequest {
        body: fs::read_to_string(file_name).unwrap(),
        headers,
    }
}

fn compare_payload_to_expected_result(file: &str, results: Vec<VcsEvent>) {
    let payload = generate_request_from_file(file);
    let subject = Github::unauthenticated();
    let mut parse_results = subject.parse_to_intermediary(payload);
    assert_eq!(parse_results.len(), 1);
    let intermediary = parse_results.pop().unwrap();
    assert_eq!(
        subject.get_repo(&intermediary),
        "https://github.com/Codertocat/Hello-World.git"
    );
    assert_eq!(results, subject.get_vcs_event(&intermediary))
}

#[test]
fn pr_created() {
    compare_payload_to_expected_result(
        "tests/data/pr_created.json",
        vec![VcsEvent::PullRequestCreate {
            number: 2,
            sha: "ec26c3e57ca3a959ca5aad62de7213c562f8c821".to_owned(),
            source_branch: "changes".to_owned(),
        }],
    )
}

#[test]
fn pr_recreated() {
    compare_payload_to_expected_result(
        "tests/data/pr_reopened.json",
        vec![VcsEvent::PullRequestCreate {
            number: 2,
            sha: "ec26c3e57ca3a959ca5aad62de7213c562f8c821".to_owned(),
            source_branch: "changes".to_owned(),
        }],
    )
}

#[test]
fn pr_updated() {
    compare_payload_to_expected_result(
        "tests/data/pr_updated.json",
        vec![VcsEvent::PullRequestUpdate {
            number: 2,
            sha: "ec26c3e57ca3a959ca5aad62de7213c562f8c821".to_owned(),
            source_branch: "changes".to_owned(),
        }],
    )
}

#[test]
fn pr_merged() {
    compare_payload_to_expected_result(
        "tests/data/pr_merged.json",
        vec![
            VcsEvent::PullRequestComplete {
                number: 2,
                merged: true,
            },
            VcsEvent::Merge {
                to_branch: "master".to_owned(),
                from_branch: "changes".to_owned(),
                sha: "f95f852bd8fca8fcc58a9a2d6c842781e32a215e".to_owned(),
            },
        ],
    )
}

#[test]
fn pr_closed() {
    compare_payload_to_expected_result(
        "tests/data/pr_closed.json",
        vec![VcsEvent::PullRequestComplete {
            number: 2,
            merged: false,
        }],
    )
}

#[test]
fn released() {
    compare_payload_to_expected_result(
        "tests/data/release.json",
        vec![VcsEvent::TagPush {
            tag: "0.0.1".to_owned(),
        }],
    )
}

#[test]
fn invalid_sig() {
    unimplemented!();
}

#[test]
fn valid_sig() {
    unimplemented!();
}
