//! Defines the high order types for saving regarding the state of a pipeline. While this
//! code does not produce a pipeline (that exists in places like webhook), it does provide
//! patterns for interacting with and operating on a pipeline.

use crate::{Application, ApprovalGroup, Stage};
use log::info;
use serde::{Deserialize, Serialize};

// TODO: This enum will be a trait and the versions will be a thing that implements
// the trait. Each of them will be implementing a "do" method or something that
// takes the state of the application and does something with the information.

//pub trait NewAction {
//    fn do(&self,  runtime: &RuntimeContext) -> impl Future;
//
//}

// An action is a task to perform given the logic for the application's configuration.
/// When evaluating each web hook event, zero or more actions are yielded from the
/// event hook. This encodes the what but not the how to perform these actions.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum Action {
    /// The app update action occurs typically on merges to master. The action,
    /// clones the code, and then runs the updates the saved state of the application.
    AppUpdate {
        /// The repo ssh path to clone down and update the job.
        repo: String,
        /// The current application state and the application to update.
        app: Application,
    },

    /// The approval action is for getting approval for the next stage in the pipeline.
    /// The approach for approval steps should be block all other steps if the approval
    /// is rejected.  
    Approval {
        /// The approval group to use to ask approval with.
        approval_group: ApprovalGroup,
        /// The stage that is getting approved.
        stage_name: String,
        /// The sha to be deployed.
        sha: String,
        /// app name that is being deployed.
        app_name: String,
    },

    /// The build action is to kick off the build job for the code and to upload the
    /// artifacts to the given location.
    Build {
        /// The ref to checkout.
        sha: String,
        /// The repo to store the code.
        repo: String,
        /// The artifact bucket to save the artifacts in.
        artifact_bucket: String,
        /// The path inside of the bucket to store the artifacts.
        artifact_folder: String,
    },

    /// The deploy job should be responsible for deploying the code to an environment.
    /// It should always follow a build in a pipeline.AsRef
    Deploy {
        /// The artifact bucket to get the artifacts from.
        artifact_bucket: String,
        /// The path inside of the bucket to store the artifacts.
        artifact_folder: String,
        /// The stage definition to load.
        stage: Stage,
    },

    /// The undeploy job should be responsible for undeploy-ing the stack from
    /// the account for the stage.
    Undeploy {
        /// The state to ignore
        stage: Stage,
    },
}

#[derive(Debug, Deserialize, Serialize)]
/// The result of performing an action.
pub enum ActionResult {
    /// The success state shows that job succeeded.
    Success,
    /// The failed state shows that job failed.
    Failed,
    /// The canceled state shows that  the action was never performed.
    Canceled,
}

/// A pipeline is a series of actions that need to be performed in order.
#[derive(Debug, Deserialize, Serialize)]
pub struct Pipeline {
    pending_actions: Vec<Action>,
    completed_actions: Vec<Action>,
    action_results: Vec<ActionResult>,
}

impl Pipeline {
    /// Generates a blank pipeline to build on.
    pub fn empty() -> Self {
        Self {
            pending_actions: Vec::new(),
            completed_actions: Vec::new(),
            action_results: Vec::new(),
        }
    }

    /// Adds a new action to the pipeline that can be performed.
    pub fn add_action(mut self, action: Action) -> Self {
        if !self.pending_actions.contains(&action) {
            self.pending_actions.push(action);
        } else {
            info!("Action{:?} is already in pipeline. No need to add", action);
        }

        self
    }

    /// Pops the next action off of the stack of actions to complete.
    pub fn pop_next_action(&mut self) -> Option<Action> {
        self.pending_actions.pop()
    }

    /// For an action in a pipeline tha was popped, this will consume that action
    /// again and take result for that action.
    pub fn complete_action(&mut self, action: Action, action_result: ActionResult) {
        self.completed_actions.push(action);
        self.action_results.push(action_result);
    }

    /// Marks all remaining steps in the pipeline as cancelled.
    pub fn cancel(&mut self) {
        while let Some(action) = self.pop_next_action() {
            self.complete_action(action, ActionResult::Canceled);
        }
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::empty()
    }
}
