//! Defines a generic way of handling web hooks from version control systems
//! that allow us to respond to events in code repositories.
//!
//! The Idea is to define a variety of entry point as well as some core logic that exists outside of any implementation of the
//! service to a cloud provider or deployment mechanism. Instead, this is a very high level operational code  and the "over all"
//! logic of the service.
//!
//! 1.) When a pull request is created, we want to store the branch, and
//! creating the pr environement if the app is set to deploy. If only set to build, just build.
//! If none is set do nothing.
//!
//! 2.) When a push is made:
//!     a.) Is it to a pr branch? If so, build / update as needed.
//!     b.) Is it a tag, deloy if there is a tag trigger that is appropriate.
//!
//! 3.) When a pull request is closed, we want to tear down the environment.
//!
use crate::{Action, Application, Pipeline, Stage, Trigger};
use regex::Regex;

/// Defines an http request subset of information that is to be processed.
#[derive(Debug)]
pub struct WebhookRequest {}

#[derive(Debug)]
/// Defines an event that is parsed from the web hook request by a
/// WebhookInterpretor.
pub struct WebhookEvent {
    event: VcsEvent,
    app: Application,
    repo: String,
}

/// Defines a standard form of event from the version control
/// system that ocurrs against the remote repository.
#[derive(Debug)]
pub enum VcsEvent {
    /// Indicates that the event is a push with a specific ref.
    Push {
        /// The ref field of the event e.g "/refs/master/HEAD"
        git_ref: String,
    },
    /// Indicates that a pull request was created.
    PullRequestCreate {
        /// The name of the branch that has the code to be merged.
        source_branch: String,
        /// The name of the branch that has the code to be merged.
        desination_branch: String,
        /// The bumber of the pr being created.
        pr_number: u32,
        /// The sha to deploy.
        current_sha: String,
    },
    /// Indicates that a pull request was completed.
    PullRequestComplete {
        /// The number of the pr being completed.
        pr_number: u32,
        /// Wether or not the pr was merged to the branch it was intended for.
        merged: bool,
    },
}

pub trait ArtifactProvider {}

/// Defines an object that interprets web hook events from a vcs
/// event web hook and converts them to a standard event.
pub trait WebhookInterpretor {
    /// The interpret_event function is responsible for examinging
    /// a request from a vcs web hook. This is intepreted into a
    /// stanard form of one or more events in the vcs.
    fn interpret_webhook_payload(&self, req: &WebhookRequest) -> Vec<WebhookEvent>;
}

fn event_to_pipeline<A: ArtifactProvider>(
    event: &WebhookEvent,
    artifact_provider: &A,
) -> Option<Pipeline> {
    let result = None;

    for trigger in &event.app.triggers {
        match trigger {
            Trigger::Pr { deploy } => {}
        }
    }

    return result;
}

/// Takes a look at the event and process it into a standard event enum.
/// Match on the enum event and evaluate the tiggers based on the application in question.
/// For each trigger that is matched, do the stuff required by that trigger as
/// another job to enqueue.
pub fn handle_web_hook_event<T: WebhookInterpretor, A: ArtifactProvider>(
    interpretor: &T,
    artifact_provider: &A,
    request: &WebhookRequest,
) -> Vec<Pipeline> {
    interpretor
        .interpret_webhook_payload(request)
        .iter()
        .map(|e| event_to_pipeline(e, artifact_provider))
        .filter_map(|o| o)
        .collect()
}
