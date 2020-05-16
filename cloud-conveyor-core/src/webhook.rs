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
use crate::{Action, Application, ArtifactProvider, Pipeline, Stage, Trigger};
use regex::Regex;
use std::collections::HashMap;

const SEMVER_REGEX: &str = "(0|(?:[1-9]\\d*))(?:\\.(0|(?:[1-9]\\d*))(?:\\.(0|(?:[1-9]\\d*)))?(?:\\-([\\w][\\w\\.\\-_]*))?)?";

/// Defines an http request subset of information that is to be processed.
#[derive(Debug)]
pub struct WebhookRequest {
    headers: HashMap<String, String>,
    body: String,
}

/// Defines an event that is parsed from the web hook request by a
/// WebhookInterpretor.
#[derive(Debug)]
pub struct WebhookEvent {
    event: VcsEvent,
    app: Application,
    repo: String,
}

/// Defines a standard form of event from the version control
/// system that ocurrs against the remote repository.
#[derive(Clone, Debug)]
pub enum VcsEvent {
    /// Indicates that the event is a push with a specific ref.
    Merge {
        /// The "to" branch that was merged into.
        to_branch: String,
        /// The "from" branch that was Merged from.
        from_branch: String,
        /// The new sha at the current branch.
        sha: String,
    },
    /// Indicates a vcs to push to a tag.
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
        /// The bumber of the pr being created.
        pr_number: u32,
        /// The sha to deploy.
        sha: String,
    },
    /// Indicates that new commits have been pushed to an existing PR.
    PullRequestUpdate {
        /// The name of the branch that has the code to be merged.
        source_branch: String,
        /// The bumber of the pr being created.
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

/// Defines an object that interprets web hook events from a vcs
/// event web hook and converts them to a standard event.
pub trait WebhookInterpretor {
    /// The interpret_event function is responsible for examinging
    /// a request from a vcs web hook. This is intepreted into a
    /// stanard form of one or more events in the vcs.
    fn interpret_webhook_payload(&self, req: &WebhookRequest) -> Vec<WebhookEvent>;
}

fn add_build_and_deploy_stages<A: ArtifactProvider>(
    artifact_provider: &A,
    pipeline: Option<Pipeline>,
    sha: &str,
    deploy_stages: Vec<Stage>,
    event: &mut WebhookEvent,
) -> Pipeline {
    let artifact_bucket = artifact_provider.get_bucket(&event.app);
    let artifact_folder = artifact_provider.get_folder(&event.app, sha);
    let build_action = Action::Build {
        repo: event.repo.clone(),
        sha: sha.to_string(),
        artifact_bucket: artifact_bucket.clone(),
        artifact_folder: artifact_folder.clone(),
    };
    let mut new_pipeline = pipeline.unwrap_or_default().add_action(build_action);

    // If the deployment should be done, we should do it. Add the step to the pipeline.
    for stage in deploy_stages {
        if let Some(approval_group) = stage.approval_group.as_ref() {
            let approval_action = Action::Approval {
                approval_group: approval_group.clone(),
                sha: sha.to_string(),
                app_name: event.app.full_name(),
                stage_name: stage.name.clone(),
            };
            new_pipeline = new_pipeline.add_action(approval_action);
        }

        let deploy_action = Action::Deploy {
            artifact_bucket: artifact_bucket.clone(),
            artifact_folder: artifact_folder.clone(),
            stage,
        };
        new_pipeline = new_pipeline.add_action(deploy_action);
    }

    new_pipeline
}

fn handle_tag_trigger<A: ArtifactProvider>(
    artifact_provider: &A,
    pipeline: Option<Pipeline>,
    event: &mut WebhookEvent,
    pattern: String,
    stages: Vec<String>,
) -> Option<Pipeline> {
    match event.event.clone() {
        VcsEvent::TagPush { tag, sha } => {
            let pattern = if pattern == "semver" {
                SEMVER_REGEX
            } else {
                pattern.as_ref()
            };

            let re = Regex::new(pattern).unwrap();
            if !re.is_match(tag.as_ref()) {
                return pipeline;
            }

            let deploy_stages: Vec<Stage> = event
                .app
                .stages
                .iter()
                .filter(|s| stages.contains(&s.name))
                .cloned()
                .collect();

            add_build_and_deploy_stages(artifact_provider, pipeline, &sha, deploy_stages, event)
                .into()
        }
        _ => pipeline,
    }
}

fn handle_merge_trigger<A: ArtifactProvider>(
    artifact_provider: &A,
    pipeline: Option<Pipeline>,
    event: &mut WebhookEvent,
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
            // invariant that all regexes will compile. So this unwrap should be okay.
            let regex = Regex::new(to_regex.as_ref()).unwrap();
            if !regex.is_match(to_branch.as_ref()) {
                return pipeline;
            }

            // If the trigger has a match regex use that or match to anything.
            // If the match is not a success, keep the current pipeline.
            let regex =
                Regex::new(from_regex.unwrap_or_else(|| String::from(".*")).as_ref()).unwrap();
            if !regex.is_match(from_branch.as_ref()) {
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
            add_build_and_deploy_stages(artifact_provider, pipeline, &sha, deploy_stages, event)
                .into()
        }
        _ => pipeline,
    }
}

fn handle_pr_trigger<A: ArtifactProvider>(
    pipeline: Option<Pipeline>,
    should_deploy: bool,
    event: &mut WebhookEvent,
    artifact_provider: &A,
) -> Option<Pipeline> {
    match event.event.clone() {
        // When a pull request is created we need to create a build job. So we will create or
        // populate the pipeline with a build step.
        VcsEvent::PullRequestCreate {
            source_branch: _,
            pr_number,
            sha,
        } => {
            // If wes should deploy, we need a new stage to be created.
            let stages = if should_deploy {
                let new_stage = Stage::from_pr_number(&event.app, pr_number);
                event.app.add_stage(new_stage.clone());
                vec![new_stage]
            } else {
                Vec::new()
            };
            let result =
                add_build_and_deploy_stages(artifact_provider, pipeline, &sha, stages, event);
            result.into()
        }
        // If the pull request is complete and there is a defined stage, then
        // we should add an undeploy job to the pipeline.
        VcsEvent::PullRequestComplete {
            pr_number,
            merged: _,
        } => {
            // Scan for the stage in the application for the PR.
            let stage = event.app.stages.iter().find(|s| s.is_for_pr(pr_number));

            // If there is a stage, we need to "undeploy" it from the appropriate
            // account.  We do not need to do any kind of final builds.
            if let Some(stage) = stage {
                let undeploy_action = Action::Undeploy {
                    stage: stage.clone(),
                };
                return pipeline
                    .unwrap_or_default()
                    .add_action(undeploy_action)
                    .into();
            }
            pipeline
        }
        // If the pull request is updated and there is a defined stage, then
        // we should add an new build / deploy  job to the pipeline.
        VcsEvent::PullRequestUpdate {
            source_branch: _,
            pr_number,
            sha,
        } => {
            // Scan for the stage in the application for the PR.
            let stage = event
                .app
                .stages
                .iter()
                .find(|s| s.is_for_pr(pr_number.clone()));

            // Add the build and deploy stages to the pipeline.
            // Of there is a stage for the pr, then we can copy that and use that
            // to deploy.
            add_build_and_deploy_stages(
                artifact_provider,
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

fn event_to_pipeline<A: ArtifactProvider>(
    event: &mut WebhookEvent,
    artifact_provider: &A,
) -> Option<Pipeline> {
    let mut result = None;

    for trigger in event.app.triggers.clone() {
        match trigger {
            Trigger::Pr { deploy } => {
                result = handle_pr_trigger(result, deploy, event, artifact_provider);
            }
            Trigger::Merge { to, from, stages } => {
                result = handle_merge_trigger(artifact_provider, result, event, to, from, stages);
            }
            Trigger::Tag { pattern, stages } => {
                result = handle_tag_trigger(artifact_provider, result, event, pattern, stages);
            }
        }
    }

    result
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
        .iter_mut()
        .map(|e| event_to_pipeline(e, artifact_provider))
        .filter_map(|o| o)
        .collect()
}
