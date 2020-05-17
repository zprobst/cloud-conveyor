//! Defines core structures and internal abstractions for Cloud Conveyor.
//! This is really an internal only crate for cloud conveyor and not meant as a standard library.
#![warn(
    missing_docs,
    rust_2018_idioms,
    missing_debug_implementations,
    intra_doc_link_resolution_failure
)]
use serde::{Deserialize, Serialize};

pub mod pipelining;
pub mod runtime;
pub mod webhook;
pub mod yaml;

// TODO: Application should be something that can be generic with things that have a "Persister<Application>" trait
// that allows us to call "save" on the application when its state has been changed by the core library functions. All
// refs to an application should be generic over the same thing then probably. Maybe there is a way to handle it a bit
// better.

// TODO: Make this docs way better.

/// Defines an group of approvers that use a single service.
/// Currently, on the slack type is supported.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum ApprovalGroup {
    /// The Slack approval pattern. When approval is needed, each of the people in the people
    /// vector should get a message that allows them to approve or deny a deployment.
    Slack {
        /// The slack handles "@zprobst" for the people who can approve with this group.
        people: Vec<String>,
    },
}

/// Defines the current status of an approval for a certain application deployment.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum ApprovalStatus {
    /// The approval request has been set to all of the participants but nobody has responded yet.
    Pending,

    /// The state of the stage when evaluating the need for approval indicated that approval was not
    /// required. This is an "allowed status"
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
    /// depending on whether or not the stage definition includes any required approvers.
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
    /// Checked if the account could potentially be a default account.
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
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Stage {
    /// The name of the stage. e.g "dev", "stage", "prod"
    pub name: String,

    ///  A group of people to approve the stage. If any.
    pub approval_group: Option<ApprovalGroup>,

    /// The reference to the account that the stage belongs to.
    pub account: Account,
}

impl Stage {
    /// Defines a new stage given the a new application, Since the reference to
    /// app is not mutable, this does not add the stage to the app. However, the stage is
    /// does contain a reference to the account.
    pub fn from_pr_number(app: &Application, number: u32) -> Self {
        let account = app
            .default_account()
            .expect("No default account on the app. This is somehow not a valid config.");
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
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Application {
    /// The org that the application is a part of. This will likely be the owner
    /// of the project on a source control platform like github.
    pub org: String,

    /// The application name of the code.  This is likely the
    pub app: String,

    /// The list of accounts in the application
    pub accounts: Vec<Account>,

    /// The internal index to the  account that is default.
    pub(crate) default_account_index: Option<usize>,

    /// The triggers of the application.
    pub stages: Vec<Stage>,

    /// The triggers of the application.
    pub triggers: Vec<Trigger>,

    /// The  different approval groups that are in the application
    pub approval_groups: Vec<ApprovalGroup>,
}

impl Application {
    /// Gets the default account for this application if there is one.
    pub fn default_account(&self) -> Option<&Account> {
        self.default_account_index.map(|i| {
            self.accounts.get(i).expect(
                "internal index for default_account pointed to a value not in the list of accounts.",
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
