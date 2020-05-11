//! Defines core structures and interal abstractions for Cloud Conveyor.
//! This is really an internal only crate for cloud conveyor and not meant as a standard library.

pub mod yaml;

/// Defines an group of approvers that use a single service.
/// Currently, on the slack type is supported.
pub enum ApprovalGroup {
    /// The Slack approval pattern. When approval is needed, each of the people in the people
    /// vector should get a message that allows them to approve or deny a deployment.
    Slack { people: Vec<String> },
}

/// Defines the current status of an approval for a certain application deployment.
pub enum ApprovalStatus {
    /// The approval request has been set to all of the particpants but nobody has responded yet.
    Pending,

    /// The state of the stage when evaluating the need for approval indicated that approval was not
    /// requred. This is an "allowed status"
    NotNeeded,

    /// This state indicates that somebody approved the deployment and stores the time and by.
    Approved { by: String },

    /// The state indicates  that somebody explicitly denied the application to continue.
    /// As a result, the application cannot be deployed.
    Rejected { by: String },

    /// The approval status when first created. This may become Pending or Not Needed
    /// depending on whether or not the stage defintion includes any required approvers.
    Unasked,
}

/// An account with a cloud provider with a cloud provider and the types to bind information'
/// for given the type of cloud provider.
#[derive(Clone)]
pub enum Account {
    Aws {
        /// The name of the aws account in question.
        name: String,
        /// The account number in aws.
        id: usize,
        /// The list of regions that should be deployed to in the application.
        regions: Vec<String>,
    },
}

/// Defines the kinds of triggers in the application that allow for
/// things to happen for user actions. For instance, pr deploys, merges to branches, etc
/// given the information provided by a source control provider such as github.
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
        to_branch: String,

        /// The pattern (if any) of branches to apply to that are related to the
        from_branch: Option<String>,

        /// The names of the stages that apply to the merge pattern.
        stage_names: Vec<String>,
    },

    /// When a tag is pushed.
    Tag {
        /// The regex pattern to apply to tags. Can also be "semver" to allow for
        /// the really complex semver regex to be used.
        pattern: String,

        /// The names of the stages that apply to the tag pattern.
        stage_names: Vec<String>,
    },
}

///  The stage of the application. This is specific an environment.
pub struct Stage {
    /// The name of the stage. e.g "dev", "stage", "prod"
    pub name: String,

    ///  A group of people to approve the stage. If any.
    pub approval_group: Option<ApprovalGroup>,

    /// The reference to the accout that the stage belongs to.
    pub account:Account,
}

impl Stage {
    /// Defines a new stage given the a new application, Since the reference to
    /// app is not mutable, this does not add the stage to the app. However, the stage is
    /// does contain a reference to the account.
    pub fn from_pr_number(app: &Application, number: u32) -> Self {
        // TODO: Figure out a better way than a panic for applications that do not have
        // a default stage; either we make it an invariant or we need to allow the user
        // to specify the account to use for prs and make sure we handle when they don't
        // have a default and don't sepcify a target account.
        let account = app
            .default_account()
            .expect("No default account on the app");
        Self {
            name: format!("pr-{}", number),
            approval_group: None,
            account: account.clone(),
        }
    }
}

/// Defines the application that is using Cloud Conveyor.
pub struct Application {
    /// The org that the application is a part of. This will likely be the owner
    /// of the project on a source control platform like github.
    pub org: String,

    /// The application name of the code.  This is likely the
    pub app: String,

    /// The  different approval groups that are in the application
    pub approval_group: Vec<ApprovalGroup>,

    /// The list of accounts in the application
    pub accounts: Vec<Account>,

    /// The triggers of the applicaiton.
    pub triggers: Vec<Trigger>,

    /// The triggers of the applicaiton.
    pub stages: Vec<Stage>,

    /// The internal index to the  account that is defualt.
    pub(crate) default_account_index: Option<usize>,
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
}

/// Creates a new Deployment for a specific application.
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
