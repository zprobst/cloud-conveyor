//! This crate defines the webhook interpreter for github webhook requests.
//!  Due to the interface for the InterpretWebhooks trait, this implementation
//! requires that all of the error cases are handled in here. If invalid request bodies are
//! sent for instance, we will simply return an empty vector of events.
//!
//! This crate also uses the standard logging pattern that the core crate uses.
#![warn(
    missing_docs,
    rust_2018_idioms,
    missing_debug_implementations,
    intra_doc_link_resolution_failure
)]

use cloud_conveyor_core::webhook::{InterpretWebhooks, VcsEvent, WebhookRequest};
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::mac::MacResult;
use crypto::sha1::Sha1;
use hex::FromHex;
use serde::Deserialize;
use serde_json::Error;

#[derive(Debug, Deserialize)]
struct BranchData {
    sha: String,
    #[serde(rename = "ref")]
    git_ref: String,
}

#[derive(Debug, Deserialize)]
struct PullRequest {
    number: u32,
    merged: bool,
    head: BranchData,
    base: BranchData,
}

#[derive(Debug, Deserialize)]
struct Repository {
    clone_url: String,
}

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
}

/// Stores information about a pr webhook payload.
#[derive(Debug, Deserialize)]
pub struct PullRequestPayload {
    repository: Repository,
    pull_request: PullRequest,
    action: String,
}

/// Stores information about a release webhook payload.
#[derive(Debug, Deserialize)]
pub struct ReleasePayload {
    repository: Repository,
    action: String,
    release: Release,
}

/// Defines the kinds of events that are relevant to the github webhook
/// interpretation.
#[derive(Debug, Deserialize)]
pub enum EventType {
    /// Release webhook payload documented [here](https://developer.github.com/webhooks/event-payloads/#release)
    Release(ReleasePayload),
    /// Pr webhook payload documented [here](https://developer.github.com/webhooks/event-payloads/#pull_request)
    Pr(PullRequestPayload),
}

/// An interface to github that allows for the interpreting of webhooks
/// with the appropriate security validations in place.
#[derive(Debug)]
pub struct Github {
    webhook_secret: Option<String>,
}

fn parse(body: String) -> Result<Vec<EventType>, Error> {
    let json = serde_json::to_value(body)?;
    let is_pull_request = json.pointer("/pull_request").is_some();
    if is_pull_request {
        let pr_data = serde_json::from_value(json)?;
        Ok(vec![EventType::Pr(pr_data)])
    } else {
        let push_data = serde_json::from_value(json)?;
        Ok(vec![EventType::Release(push_data)])
    }
}

impl Github {
    fn authenticate(&self, payload: &str, signature: &[u8]) -> bool {
        if let Some(webhook_secret) = &self.webhook_secret {
            // Github gives you an HMAC code for the payload. Match it.
            // https://developer.github.com/webhooks/securing/
            match Vec::from_hex(signature) {
                Ok(signature_bytes) => {
                    let mut mac = Hmac::new(Sha1::new(), &webhook_secret.as_bytes());
                    mac.input(&payload.as_bytes());
                    mac.result() == MacResult::new(&signature_bytes)
                }
                Err(_) => false,
            }
        } else {
            // If the user has not specified a web hooks secret, we cannot verify it.
            // Potentially we should not allow it to be unset, but if you are experimenting,
            // it might be a hinderance.
            true
        }
    }
}

impl InterpretWebhooks for Github {
    type Intermediary = EventType;

    fn parse_to_intermediary(&self, req: WebhookRequest) -> Vec<Self::Intermediary> {
        // TODO: Validate the github signature.
        let signature_header = &req.headers["X-Hub-Signature"];
        let sans_prefix = signature_header[5..signature_header.len()].as_bytes();

        if self.authenticate(&req.body, sans_prefix) {
            match parse(req.body) {
                Ok(results) => results,
                Err(_) => Vec::with_capacity(0),
            }
        } else {
            Vec::with_capacity(0)
        }
    }
    fn get_vcs_event(&self, payload: &Self::Intermediary) -> Vec<VcsEvent> {
        match payload {
            EventType::Release(release_data) => match release_data.action.as_ref() {
                // We only care about the published action. When a release is published, we
                // need to indicate  a tag push. This is a little more convenient than doing a tag
                // push but does not allow from releases from the command line.
                "published" => vec![VcsEvent::TagPush {
                    tag: release_data.release.tag_name.clone(),
                }],
                _ => Vec::with_capacity(0),
            },
            EventType::Pr(pr_data) => match pr_data.action.as_ref() {
                // If a PR is opened, we will want to start a pr create. If a PR is re-opened,
                // we should treat the pr the same way because the infra has been torn down
                // like nothing has ever happened.
                "opened" | "reopened" => vec![VcsEvent::PullRequestCreate {
                    number: pr_data.pull_request.number,
                    sha: pr_data.pull_request.head.sha.clone(),
                    source_branch: pr_data.pull_request.head.git_ref.clone(),
                }],

                // synchronize is used to indicate a change in the branch state or
                // that there was a push to the source branch. As such, we need  to
                // tell CC that the was a na update to the PR.
                "synchronize" => vec![VcsEvent::PullRequestUpdate {
                    number: pr_data.pull_request.number,
                    sha: pr_data.pull_request.head.sha.clone(),
                    source_branch: pr_data.pull_request.head.git_ref.clone(),
                }],

                // When a PR is closed, we will want to start at most two pipelines.
                // One to teardown the pr build and a second to trigger a merge job.
                // because of this, this pull request payload is really sending us information
                //about two events if the pr was merged. Therefore, we need to return
                // two events if it was.
                "closed" => {
                    let pr_closed = VcsEvent::PullRequestComplete {
                        merged: pr_data.pull_request.merged,
                        number: pr_data.pull_request.number,
                    };
                    if pr_data.pull_request.merged {
                        vec![
                            pr_closed,
                            VcsEvent::Merge {
                                to_branch: pr_data.pull_request.base.git_ref.clone(),
                                from_branch: pr_data.pull_request.head.git_ref.clone(),
                                sha: pr_data.pull_request.head.sha.clone(),
                            },
                        ]
                    } else {
                        vec![pr_closed]
                    }
                }
                _ => Vec::with_capacity(0),
            },
        }
    }
    fn get_repo<'a>(&self, payload: &'a Self::Intermediary) -> &'a str {
        match payload {
            EventType::Release(push_data) => &push_data.repository.clone_url,
            EventType::Pr(pr_data) => &pr_data.repository.clone_url,
        }
    }
}
