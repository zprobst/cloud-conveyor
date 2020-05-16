//! Defines core structures and interal abstractions for Cloud Conveyor.
//! This is really an internal only crate for cloud conveyor and not meant as a standard library.
#![warn(
    missing_docs,
    rust_2018_idioms,
    missing_debug_implementations,
    intra_doc_link_resolution_failure
)]
use log::info;
use serde::{Deserialize, Serialize};

pub mod webhook;
pub mod yaml;

/// Defines an group of approvers that use a single service.
/// Currently, on the slack type is supported.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum ApprovalGroup {
    /// The Slack approval pattern. When approval is needed, each of the people in the people
    /// vector should get a message that allows them to approve or deny a deployment.
    Slack {
        /// The slack handles "@zprobst" for the people who can approve with this group.
        people: Vec<String>,
    },
}

/// Defines the current status of an approval for a certain application deployment.
#[derive(Debug, Deserialize, PartialEq)]
pub enum ApprovalStatus {
    /// The approval request has been set to all of the particpants but nobody has responded yet.
    Pending,

    /// The state of the stage when evaluating the need for approval indicated that approval was not
    /// requred. This is an "allowed status"
    NotNeeded,

    /// This state indicates that somebody approved the deployment and stores the time and by.
    Approved {
        /// The handle of the person who approved.
        by: String,
    },

    /// The state indicates  that somebody explicitly denied the application to continue.
    /// As a result, the application cannot be deployed.
    Rejected {
        /// The handle of the person who approved.  
        by: String,
    },

    /// The approval status when first created. This may become Pending or Not Needed
    /// depending on whether or not the stage defintion includes any required approvers.
    Unasked,
}

/// An account with a cloud provider with a cloud provider and the types to bind information'
/// for given the type of cloud provider.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Account {
    /// The name of the aws account in question.
    pub name: String,
    /// The account number in aws.
    pub id: usize,
    /// The list of regions that should be deployed to in the application.
    pub regions: Vec<String>,
}

impl Account {
    /// Checked if the accout could potentially be a default account.
    pub fn is_candidate_for_default(&self) -> bool {
        self.is_named("default")
    }

    ///  Check if the account is named a certain thing.
    pub fn is_named(&self, name: &str) -> bool {
        name == self.name
    }
}

/// Defines the kinds of triggers in the application that allow for
/// things to happen for user actions. For instance, pr deploys, merges to branches, etc
/// given the information provided by a source control provider such as github.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Trigger {
    /// PR Builds an deploys. When they occur, new temporary stacks are created and updated
    /// for the life of the pull request (or similar notion depending on the source provider). When
    // Builds are implicitly true if deploys are true.
    Pr {
        /// Whether or not a temporary stack should be created, updated, deleted in line
        /// with the lifetime of the PR.
        deploy: bool,
    },

    /// When a merge is made to a branch. Optional filter on what kind of branch
    /// the merge came from as well.
    Merge {
        ///The pattern of branches to apply to that are getting merged into; e.g master.
        /// Can be any valid regular expression.
        to: String,

        /// The pattern (if any) of branches to apply to that are related to the
        from: Option<String>,

        /// The names of the stages that apply to the merge pattern.
        #[serde(rename = "deploy")]
        stages: Vec<String>,
    },

    /// When a tag is pushed.
    Tag {
        /// The regex pattern to apply to tags. Can also be "semver" to allow for
        /// the really complex semver regex to be used.
        pattern: String,

        /// The names of the stages that apply to the tag pattern.
        #[serde(rename = "deploy")]
        stages: Vec<String>,
    },
}

///  The stage of the application. This is specific an environment.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Stage {
    /// The name of the stage. e.g "dev", "stage", "prod"
    pub name: String,

    ///  A group of people to approve the stage. If any.
    pub approval_group: Option<ApprovalGroup>,

    /// The reference to the accout that the stage belongs to.
    pub account: Account,
}

impl Stage {
    /// Defines a new stage given the a new application, Since the reference to
    /// app is not mutable, this does not add the stage to the app. However, the stage is
    /// does contain a reference to the account.
    pub fn from_pr_number(app: &Application, number: u32) -> Self {
        let account = app.default_account().expect(
            "No default account on the app. We should not have onboarded or updated the config.",
        );
        Self {
            name: format!("pr-{}", number),
            approval_group: None,
            account: account.clone(),
        }
    }

    /// Determines if a branch is for a PR or not.
    pub fn is_for_pr(&self, pr_number: u32) -> bool {
        self.name == format!("pr-{}", pr_number)
    }
}

/// Defines the application that is using Cloud Conveyor.
#[derive(Debug, Deserialize, PartialEq)]
pub struct Application {
    /// The org that the application is a part of. This will likely be the owner
    /// of the project on a source control platform like github.
    pub org: String,

    /// The application name of the code.  This is likely the
    pub app: String,

    /// The list of accounts in the application
    pub accounts: Vec<Account>,

    /// The internal index to the  account that is defualt.
    pub(crate) default_account_index: Option<usize>,

    /// The triggers of the applicaiton.
    pub stages: Vec<Stage>,

    /// The triggers of the applicaiton.
    pub triggers: Vec<Trigger>,

    /// The  different approval groups that are in the application
    pub approval_groups: Vec<ApprovalGroup>,
}

impl Application {
    /// Gets the default account for this application if there is one.
    pub fn default_account(&self) -> Option<&Account> {
        self.default_account_index.map(|i| {
            self.accounts.get(i).expect(
                "interal index for default_account pointed to a value not in the list of accounts.",
            )
        })
    }

    /// Adds a new stage to the application.
    pub fn add_stage(&mut self, stage: Stage) {
        self.stages.push(stage)
    }

    /// Returns the full name of the application org/app-name
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.org, self.app)
    }
}

/// Creates a new Deployment for a specific application.
#[derive(Debug, PartialEq)]
pub struct Deployment<'trigger> {
    ///  The application that is being deployed.
    pub app: Application,

    /// The stage of the application that is being deployed,
    pub stage: Stage,

    /// The Code Sha that is being deployed.
    pub sha: String,

    /// A flag indicating the deployment is currently running.
    pub is_running: bool,

    /// A flag indicating the deployment was a success. If the deployment
    /// is still running, this value will be false. That does not mean it is unsucessful.
    pub was_success: bool,

    /// The storage bucket where the assets for this deployment are currently at.
    pub artifact_bucket: String,

    /// The path in the above bucket that will be a folder containing all artifacts.
    pub artifact_path: String,

    /// The trigger that caused the deployment.
    pub trigger: &'trigger Trigger,

    /// The triggerer of the deployment. This is the person that caused the event to happen
    ///  in source control.
    pub caused_by: String,

    ///  The current state of the approval for this deployment.
    pub approval_status: ApprovalStatus,
}

/// An action is a task to perform given the logic for the application's configuration.
/// When evaluating each web hook event, zero or more actions are yielded from the
/// event hook. This encodes the what but not the how to perform these actions.
#[derive(Debug, PartialEq)]
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

    /// The deploy job should be responsible for deploying the code to an environemnt.
    /// It should always follow a build in a pipeline.AsRef
    Deploy {
        /// The artifact bucket to get the artifacts from.
        artifact_bucket: String,
        /// The path inside of the bucket to store the artifacts.
        artifact_folder: String,
        /// The stage definition to load.
        stage: Stage,
    },

    /// The undeploy job should be responsible for undeploying the stack from
    /// the account for the stage.
    Undeploy {
        /// The state to ignore
        stage: Stage,
    },
}

//// WIP Code /////

#[derive(Debug)]
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
#[derive(Debug)]
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

/// The artifact provider is something that adds buckets and
/// folders to buckets to provide locations to store assets.
pub trait ArtifactProvider {
    /// The bucket for the application.
    fn get_bucket(&self, app: &Application) -> String;
    /// The folder for the application and current sha.
    fn get_folder(&self, app: &Application, git_sha: &str) -> String;
}
