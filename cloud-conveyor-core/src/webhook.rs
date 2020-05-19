//! Defines a generic way of handling web hooks from version control systems
//! that allow us to respond to events in code repositories.
//!
//! These events are processed against the corresponding application's triggers and
//! determines if a pipeline needs to be executed, and if so, what steps are in that pipeline.
//!  More concretely, we process the following types here:
//!
//! 1.) When a pull request is created,  updated, or deleted and the application has pr builds
//! enabled, we will want to perform the appropriate actions. This will likely be a build and
//! deploy of that code to a ephemeral environment lasting the lifetime of the PR.
//!
//!  2.) When a tag is pushed and the application has a tag trigger who's pattern matches the tag
//!  that was pushed, we will want to build and deploy the code to the environment list
//! that exists in the aforementioned trigger.
//!
//!  3.) When a branch is merged, and the application has a merge trigger who's branch name pattern
//! matches the name of the branch merged into (and optionally the same for the source branch) then
//! we will want to build and deploy the code to the environment list  that exists in the aforementioned trigger.

// DEV NOTE: There are many locations where we "just clone" stuff. This _seems_ like it has to be a
// necessary evil. We have to have a lot of owned information in structures because many of the types in the
// core library implement serialize and deserialize for downstream crates.

use crate::pipelining::{Approval, Build, Deploy, Pipeline, Teardown};
use crate::runtime::RuntimeContext;
use crate::{Application, Stage, Trigger};
use log::info;
use regex::Regex;
use std::collections::HashMap;

const SEMVER_REGEX: &str = "(0|(?:[1-9]\\d*))(?:\\.(0|(?:[1-9]\\d*))(?:\\.(0|(?:[1-9]\\d*)))?(?:\\-([\\w][\\w\\.\\-_]*))?)?";

/// Defines a simple object that roughly scaffolds some of the information in an
/// HTTP Post request. This module assumes that the underlying hook system for
/// the vcs service in question uses that scheme to deliver messages.
#[derive(Debug)]
pub struct WebhookRequest {
    /// The headers of the request.
    pub headers: HashMap<String, String>,
    /// The http body of the request.
    pub body: String,
}

/// Defines a standard form of event from the version controls system that occurs against the remote repository.
/// This enum is certainly not a
#[derive(Clone, Debug)]
pub enum VcsEvent {
    /// Indicates when one branch is merged into another.
    Merge {
        /// The "to" branch that was merged into.
        to_branch: String,
        /// The "from" branch that was Merged from.
        from_branch: String,
        /// The new sha at the current branch.
        sha: String,
    },
    /// Indicates when a new tag was pushed to the repository.
    TagPush {
        /// The tag name to push.
        tag: String,
        /// The sha that is attached to command.
        sha: String,
    },
    /// Indicates that a pull request was created.
    PullRequestCreate {
        /// The name of the branch that has the code to be merged.
        source_branch: String,
        /// The number of the pr being created.
        pr_number: u32,
        /// The sha to deploy.
        sha: String,
    },
    /// Indicates that new commits have been pushed to an existing PR.
    PullRequestUpdate {
        /// The name of the branch that has the code to be merged.
        source_branch: String,
        /// The number of the pr being created.
        pr_number: u32,
        /// The sha to deploy.
        sha: String,
    },
    /// Indicates that a pull request was completed.
    PullRequestComplete {
        /// The number of the pr being completed.
        pr_number: u32,
        /// Wether or not the pr was merged to the branch it was intended for.
        merged: bool,
    },
}

/// Defines a parsed event that came from a web request hook
#[derive(Debug)]
pub struct WebhookEvent<'application> {
    event: VcsEvent,
    app: &'application mut Application,
    repo: String,
}

/// Defines a trait for something that takes [WebhookRequest](struct.WebhookRequest.html) objects and
/// parses them for any version control events as specified in [VcsEvent](enum.VcsEvent.html). Since the
/// payloads for various different vcs providers (github, bitbucket, etc.) are different,
/// we need this to be a trait that can be implemented for different providers in a separate crate.
///
/// The approach to this is a forced compartmentalization of the responsibilities. You will define several functions.
/// One that parses the event into an intermediary form of your choosing and two other methods that consume
/// references to that intermediary type and provide information from that type to the internals of cloud conveyor
/// that are needed in order to perform the operations required by the module level documentation [here](index.html).
pub trait InterpretWebhooks {
    /// Intermediary type  that you parse the raw web hook events into.
    type Intermediary;

    /// Parses the web hook request as one or more events that are indicated with your intermediary
    /// type declaration. That means the size of the vec returned must match the number of individual
    /// events that that webhook contained. This is done to support web hooks that batch events into
    /// groups to reduce calls. If there is only one event per call, return a vec of size one.
    ///
    /// If your parsing has errors, this likely means that, assuming the implementation is
    /// correct,  the data is invalid and by definition does not supply any kind of information
    /// to be processed. As such, errors should be handled and remapped as an empty vec.
    fn parse_to_intermediary(&self, req: &WebhookRequest) -> Vec<Self::Intermediary>;

    /// This will take the intermediary type and return an option of a vcs event. If the event
    /// described by the intermediary object does not relate to any one of the [VcsEvent](enum.VcsEvent.html)
    /// types, then None can be returned. Items that return none are dropped from the pipeline.
    fn get_vcs_event(&self, intermediary: &Self::Intermediary) -> Option<VcsEvent>;

    /// Gets the repo of the event. This function, unlike the others in this trait cannot return an
    /// option because it does not make sense to have a repository event that does not have a
    /// repository. This function takes the intermediary type and return and returns a string
    /// with which defines the git url for the repo.
    fn get_repo(&self, intermediary: &Self::Intermediary) -> &str;

    /// The high order function that converts payloads from a webhook to a serialized and standard
    /// form for processing in the rest of the cloud conveyor pipelining code.
    fn interpret_webhook_payload<'application>(
        &self,
        req: &WebhookRequest,
        runtime: &'application RuntimeContext,
    ) -> Vec<WebhookEvent<'application>> {
        let mut result = Vec::new();

        for inter in self.parse_to_intermediary(req) {
            let repo = self.get_repo(&inter);
            let maybe_app = runtime.load_application_from_repo(repo);
            let maybe_vcs_event = self.get_vcs_event(&inter);

            if let Some(app) = maybe_app {
                if let Some(event) = maybe_vcs_event {
                    result.push(WebhookEvent {
                        repo: repo.to_string(),
                        app,
                        event,
                    });
                }
            }
        }

        result
    }
}

fn add_build_and_deploy_stages(
    pipeline: Option<Pipeline>,
    sha: &str,
    deploy_stages: Vec<Stage>,
    event: &mut WebhookEvent<'_>,
) -> Pipeline {
    let build_action = Build::new(event.repo.clone(), sha.to_string());
    info!(
        "Pushing build action for  for sha {:?} with action {:?} ",
        sha, build_action
    );
    let mut new_pipeline = pipeline
        .unwrap_or_default()
        .add_action(Box::new(build_action));

    // If the deployment should be done, we should do it. Add the step to the pipeline.
    for stage in deploy_stages {
        if let Some(approval_group) = &stage.approval_group {
            let approval_action = Approval {
                approval_group: approval_group.clone(),
                sha: sha.to_string(),
                app_name: event.app.full_name(),
                stage_name: stage.name.clone(),
            };
            info!(
                "Pushing approval required  for stage {:?} with action {:?} ",
                stage, approval_action
            );
            new_pipeline = new_pipeline.add_action(Box::new(approval_action));
        }

        let deploy_action = Deploy::new(stage.clone(), event.repo.clone(), sha.to_string());
        info!(
            "Pushing deploy  action for stage {:?} with action {:?}",
            stage, deploy_action
        );
        new_pipeline = new_pipeline.add_action(Box::new(deploy_action));
    }

    new_pipeline
}

fn handle_tag_trigger(
    pipeline: Option<Pipeline>,
    event: &mut WebhookEvent<'_>,
    pattern: String,
    stages: Vec<String>,
) -> Option<Pipeline> {
    match event.event.clone() {
        VcsEvent::TagPush { tag, sha } => {
            let pattern = if pattern == "semver" {
                SEMVER_REGEX
            } else {
                &pattern
            };

            let re = Regex::new(pattern).unwrap();
            if !re.is_match(&tag) {
                info!("Tag {:?} does not follow the pattern  {:?}", tag, pattern);
                return pipeline;
            }

            let deploy_stages: Vec<Stage> = event
                .app
                .stages
                .iter()
                .filter(|s| stages.contains(&s.name))
                .cloned()
                .collect();

            add_build_and_deploy_stages(pipeline, &sha, deploy_stages, event).into()
        }
        _ => pipeline,
    }
}

fn handle_merge_trigger(
    pipeline: Option<Pipeline>,
    event: &mut WebhookEvent<'_>,
    to_regex: String,
    from_regex: Option<String>,
    stages: Vec<String>,
) -> Option<Pipeline> {
    match event.event.clone() {
        VcsEvent::Merge {
            to_branch,
            from_branch,
            sha,
        } => {
            // If the merge is to a branch that matches the to_regex, we are good.
            // If not, we can abandon the version. We are going to consider it an
            // invariant that all regular expressions will compile. So this unwrap should be okay.
            let regex = Regex::new(&to_regex).unwrap();
            if !regex.is_match(&to_branch) {
                info!(
                    "Branch {:?} does not match pattern {:?}",
                    to_branch, to_regex
                );
                return pipeline;
            }

            // If the trigger has a match regex use that or match to anything.
            // If the match is not a success, keep the current pipeline.
            let regex = Regex::new(&from_regex.unwrap_or_else(|| String::from(".*"))).unwrap();
            if !regex.is_match(&from_branch) {
                info!(
                    "Branch {:?} does not match pattern {:?}",
                    from_branch, regex
                );
                return pipeline;
            }

            // Now we need to find the stages in the app by the names.
            let deploy_stages: Vec<Stage> = event
                .app
                .stages
                .iter()
                .filter(|s| stages.contains(&s.name))
                .cloned()
                .collect();

            // Now that we have all of the stages, enqueue a build job and
            // a deploy job. The deploy jobs need be in the same order
            // as the vec for the stage names. That is the pattern for pipelining
            // envs.
            add_build_and_deploy_stages(pipeline, &sha, deploy_stages, event).into()
        }
        _ => pipeline,
    }
}

fn handle_pr_trigger(
    pipeline: Option<Pipeline>,
    should_deploy: bool,
    event: &mut WebhookEvent<'_>,
) -> Option<Pipeline> {
    match event.event.clone() {
        // When a pull request is created we need to create a build job. So we will create or
        // populate the pipeline with a build step.
        VcsEvent::PullRequestCreate { pr_number, sha, .. } => {
            info!(
                "Creating PR {:?} with deploy {:?}",
                pr_number, should_deploy
            );

            // If wes should deploy, we need a new stage to be created.
            let stages = if should_deploy {
                let new_stage = Stage::from_pr_number(&event.app, pr_number);
                event.app.add_stage(new_stage.clone());
                vec![new_stage]
            } else {
                Vec::new()
            };
            let result = add_build_and_deploy_stages(pipeline, &sha, stages, event);
            result.into()
        }
        // If the pull request is complete and there is a defined stage, then
        // we should add an undeploy job to the pipeline.
        VcsEvent::PullRequestComplete { pr_number, .. } => {
            info!("Completing PR {:?}", pr_number);

            // Scan for the stage in the application for the PR.
            let stage = event.app.stages.iter().find(|s| s.is_for_pr(pr_number));

            // If there is a stage, we need to "undeploy" it from the appropriate
            // account.  We do not need to do any kind of final builds.
            if let Some(stage) = stage {
                let teardown = Teardown::new(stage.clone(), event.repo.clone());
                return pipeline
                    .unwrap_or_default()
                    .add_action(Box::new(teardown))
                    .into();
            }
            pipeline
        }
        // If the pull request is updated and there is a defined stage, then
        // we should add an new build / deploy  job to the pipeline.
        VcsEvent::PullRequestUpdate { pr_number, sha, .. } => {
            // Scan for the stage in the application for the PR.
            info!("Updating PR {:?}", pr_number);
            let stage = event.app.stages.iter().find(|s| s.is_for_pr(pr_number));

            // Add the build and deploy stages to the pipeline.
            // Of there is a stage for the pr, then we can copy that and use that
            // to deploy.
            add_build_and_deploy_stages(
                pipeline,
                &sha,
                match stage {
                    Some(s) => vec![s.clone()],
                    None => Vec::new(),
                },
                event,
            )
            .into()
        }
        _ => pipeline,
    }
}

fn event_to_pipeline(event: &mut WebhookEvent<'_>) -> Option<Pipeline> {
    let mut result = None;

    for trigger in event.app.triggers.clone() {
        match trigger {
            Trigger::Pr { deploy } => {
                info!(
                    "Processing PR Trigger with deploy set to {:?} for app {:?}",
                    deploy,
                    event.app.full_name()
                );
                result = handle_pr_trigger(result, deploy, event);
            }
            Trigger::Merge { to, from, stages } => {
                info!(
                    "Processing merge trigger from {:?} to {:?} for app {:?}",
                    from,
                    to,
                    event.app.full_name()
                );
                result = handle_merge_trigger(result, event, to, from, stages);
            }
            Trigger::Tag { pattern, stages } => {
                info!(
                    "Processing tag trigger with pattern {:?} for app {:?}",
                    pattern,
                    event.app.full_name()
                );
                result = handle_tag_trigger(result, event, pattern, stages);
            }
        }
    }

    result
}

/// Given a request to a webhook endpoint, that request is passed to the specific
/// implementation of the [WebhookInterpreter](trait.WebhookInterpreter.html) trait. That trait object will
/// process teh request for any [VcsEvent](enum.VcsEvent.html) that we care about. This will invoke operations
/// to compare those events against the application's triggers for anything that needs to be done.
pub fn handle_web_hook_event<T: InterpretWebhooks>(
    interpreter: &T,
    runtime: &mut RuntimeContext,
    request: &WebhookRequest,
) -> Vec<Pipeline> {
    interpreter
        .interpret_webhook_payload(request, runtime)
        .iter_mut()
        .map(event_to_pipeline)
        .filter_map(|o| o)
        .collect()
}
