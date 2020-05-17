//! Defines the high order types for saving regarding the state of a pipeline. While this
//! code does not produce a pipeline (that exists in places like webhook), it does provide
//! patterns for interacting with and operating on a pipeline.

// TODO: We are probably going to need to serialize the pipeline.
// Probably this is the solution: https://stackoverflow.com/questions/50021897

use crate::runtime::RuntimeContext;
use crate::{ApprovalGroup, Stage};
use serde::{Deserialize, Serialize};
use std::boxed::Box;
use std::fmt::Debug;

/// The result of performing an action via the [Perform](trait.Perform.html) trait.
#[derive(Debug, Deserialize, Serialize)]
pub enum ActionResult {
    /// The success state shows that job succeeded. This means that everything in the job
    /// went to plan and following steps in the pipeline can occur.
    Success,
    /// The failed state shows that job failed and that the rest of the pipeline cannot continue
    /// and needs to be cancelled.
    Failed,
    /// The canceled state shows that  the action was never performed because a previous action
    /// in the pipeline failed. This should NOT be used in most cases for the return
    /// from [get_result](trait.Perform.html#tymethod.get_result) in the [Perform](trait.Perform.html) trait.
    Canceled,
}

/// Defines an abstract "action" in the code that can be done by "performing" it.
/// Since cloud conveyor does jobs in an external context from the main application,
/// we end up with an API that is similar to implementing a future - but not exactly the same.
///
/// The first part is [start](#tymethod.start) which will be responsible for kicking off the job
/// in an external context. This will likely involve something like an async api call to api to
/// start a job. For instance, if it is a build job, we will probably fire the call to start building the
/// project with the parameters specified by the  type of action that it is.
///
/// The second part is the [is_done](#tymethod.is_done) method which is our polling mechanism.
/// When there is a currently operating action, we need to determine if that thing is done. If it is
/// not, we will wait more. If it is, we can fetch the result through the [get_result](#tymethod.get_result)
/// function; the third and final method on the struct.
pub trait Perform: Debug {
    /// Does the work required to start the job in some sort of external context.
    fn start(&mut self, ctx: &RuntimeContext<'_, '_>) -> Result<(), ()>;

    /// Does the work required to see if the job, in the external context, is done (regardless of success or fail).
    /// If it is done, Ok(true) should be returned. If not Ok(false).
    fn is_done(&mut self, ctx: &RuntimeContext<'_, '_>) -> Result<bool, ()>;

    /// Gets the final state of the job and returns a [ActionResult](enum.ActionResult.html). For information regarding
    /// when to return what version of [ActionResult](enum.ActionResult.html), see the docs on [ActionResult](enum.ActionResult.html).
    fn get_result(&self, ctx: &RuntimeContext<'_, '_>) -> ActionResult;
}

// TODO: FIll out the spec for this type.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct AppUpdate(String);

/// The Approval action is responsible for managing the need to get approval from a human prior to
/// continuing through the pipeline. It does so by implementing the [Perform](trait.Perform.html) trait.
///
/// For example,  this is used is many of the [Triggers](enum.Trigger.html) use the Approval action. Merges
/// to a branch and pushes of a tag invoke deployments to [Stages](../struct.Stage.html) that may more many
/// require approval by specifying an [ApprovalGroup](../struct.ApprovalGroup.html) and thus the creation of an
/// approval action would only occur if that group is set.
///
///  ```rust
///  let approve = Approval {
///      approval_group: ApprovalGroup::Slack{ people: vec![] },
///      stage_name: "prod".to_string(),
///      sha: "cda888fd29a23fdb2d905e4ab6cf50230ce4c37b".to_string(),
///      app_name: "cloud_conveyor".to_string()
///  };
/// ```
///  See the implementation for the [webhook](../webhook/index.html) module for
/// more information on its consumption.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Approval {
    /// The approval group to use to ask approval with.
    pub approval_group: ApprovalGroup,
    /// The stage that is getting approved.
    pub stage_name: String,
    /// The sha to be deployed.
    pub sha: String,
    /// app name that is being deployed.
    pub app_name: String,
}

impl Perform for Approval {
    fn start(&mut self, _: &RuntimeContext<'_, '_>) -> std::result::Result<(), ()> {
        todo!()
    }
    fn is_done(&mut self, _: &RuntimeContext<'_, '_>) -> std::result::Result<bool, ()> {
        todo!()
    }
    fn get_result(&self, _: &RuntimeContext<'_, '_>) -> ActionResult {
        todo!()
    }
}

/// The Build action is responsible for managing the build of the application source into zero or more
/// artifacts that are stored in a pre-determined location. It does so by implementing the [Perform](trait.Perform.html) trait.
///
/// For example,  this is used is many of the [Triggers](enum.Trigger.html) use the deploy stage. Merges
/// to a branch, pushes of a tag, and pr creates or updates can all invoke the creation of builds jobs prior to
/// their [deploy](struct.Deploy.html) job(s).
///
/// Broadly speaking, the build type is an action that will invoke the building of the
/// application's source using the [runtime](../runtime/index.html) configuration via
/// the [RuntimeContext](../runtime/struct.RuntimeContext.html).
///
///  ```rust
///  let deploy = Build {
///      sha:  "cda888fd29a23fdb2d905e4ab6cf50230ce4c37b".to_string()
///      repo:  "git@github.com:resilient-vitality/cloud-conveyor.git".to_string(),
///      artifact_bucket: "my_bucket".to_string(),
///      artifact_folder: "org/app/my-code-sha/".to_string()
///  };
/// ```
///  See the implementation for the [webhook](../webhook/index.html) module for
/// more information on its consumption.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Build {
    /// The ref to checkout.
    pub sha: String,
    /// The repo to check the code out from.
    pub repo: String,
    /// The storage bucket to save the artifacts in. This value is semantically
    /// a string but the value is only relevant to the [ProvideArtifact](../runtime/trait.ProvideArtifact.html)
    /// implementation stored in the [RuntimeContext](../runtime/struct.RuntimeContext.html).
    pub artifact_bucket: String,
    /// The artifact path inside of the storage bucket to save the artifacts in. This value is semantically
    /// a string but the value is only  relevant to the [ProvideArtifact](../runtime/trait.ProvideArtifact.html)
    /// implementation stored in the [RuntimeContext](../runtime/struct.RuntimeContext.html).
    pub artifact_folder: String,
}

impl Perform for Build {
    fn start(&mut self, _: &RuntimeContext<'_, '_>) -> std::result::Result<(), ()> {
        todo!()
    }
    fn is_done(&mut self, _: &RuntimeContext<'_, '_>) -> std::result::Result<bool, ()> {
        todo!()
    }
    fn get_result(&self, _: &RuntimeContext<'_, '_>) -> ActionResult {
        todo!()
    }
}

/// The Deploy action is responsible for managing the updating or creation of an infrastructure stack
/// for a given application. It does so by implementing the [Perform](trait.Perform.html) trait.
///
/// For example,  this is used is many of the [Triggers](enum.Trigger.html) use the deploy stage. Merges
/// to a branch, pushes of a tag, and pr creates or updates can all invoke the creation of deploy jobs.
/// The pattern for which that occurs is dependant on the inner type of the trigger.
///
/// Broadly speaking, the deploy type is an action will invoke the creating or updating of a stack using
/// the infrastructure controller managed by the [runtime](../runtime/index.html) via the [RuntimeContext](../runtime/struct.RuntimeContext.html).
///
///  ```rust
///  let deploy = Deploy {
///      stage:  /* Some stage */,
///      repo:  "git@github.com:resilient-vitality/cloud-conveyor.git".to_string(),
///      artifact_bucket: "my_bucket".to_string(),
///      artifact_folder: "org/app/my-code-sha/".to_string()
///  };
/// ```
///  See the implementation for the [webhook](../webhook/index.html) module for
/// more information on its consumption.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Deploy {
    /// The artifact bucket to get the artifacts from.
    pub artifact_bucket: String,
    /// The path inside of the bucket to store the artifacts.
    pub artifact_folder: String,
    /// The stage definition to load.
    pub stage: Stage,
    /// The repo of the application in question. The repo is used to capture what application the stage
    /// belongs to with storing the application or a reference to it.
    pub repo: String,
}

impl Perform for Deploy {
    fn start(&mut self, _: &RuntimeContext<'_, '_>) -> std::result::Result<(), ()> {
        todo!()
    }
    fn is_done(&mut self, _: &RuntimeContext<'_, '_>) -> std::result::Result<bool, ()> {
        todo!()
    }
    fn get_result(&self, _: &RuntimeContext<'_, '_>) -> ActionResult {
        todo!()
    }
}

/// The Undeploy action is responsible for managing the deletion of stacks that are no longer required.
/// It does so by implementing the [Perform](trait.Perform.html) trait.
///
/// For example, the most common usage of this is when a pull request is closed. For applications that
/// have enabled both pr builds and deploys, temporary application stacks are stood up to allow for
/// swift iteration on changes made in that PR. However, when the pr is closed, it would not be good
/// to leave that stack laying around. So the undeploy job will delete that stage using the appropriate
/// infrastructure tooling for the stage provided.
///
///  ```rust
///  let undeploy = Undeploy {
///      stage:  /* Some stage */,
///      repo:  "git@github.com:resilient-vitality/cloud-conveyor.git".to_string()
///  };
/// ```
///  See the implementation for the [webhook](../webhook/index.html) module for
/// more information on its consumption.
///
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Undeploy {
    /// The stage to remove from the application. This stage will be deleted when the application
    pub stage: Stage,
    /// The repo of the application in question. The repo is used to capture what application the stage
    /// belongs to with storing the application or a reference to it.
    pub repo: String,
}

impl Perform for Undeploy {
    fn start(&mut self, _: &RuntimeContext<'_, '_>) -> std::result::Result<(), ()> {
        todo!()
    }
    fn is_done(&mut self, _: &RuntimeContext<'_, '_>) -> std::result::Result<bool, ()> {
        todo!()
    }
    fn get_result(&self, _: &RuntimeContext<'_, '_>) -> ActionResult {
        todo!()
    }
}

/// A pipeline is a series of actions that need to be performed in order. It is like a stack, responsible
/// for popping and pushing actions that implement the [Perform](trait.Perform.html) trait.
#[derive(Debug)]
pub struct Pipeline {
    pending_actions: Vec<Box<dyn Perform>>,
    completed_actions: Vec<Box<dyn Perform>>,
    action_results: Vec<ActionResult>,
}

impl Pipeline {
    /// Generates a blank pipeline to build on. This pipeline has no
    /// pending or completed actions.
    pub fn empty() -> Self {
        Self {
            pending_actions: Vec::new(),
            completed_actions: Vec::new(),
            action_results: Vec::new(),
        }
    }

    /// Adds a new action to the pipeline that can be performed.
    pub fn add_action(mut self, action: Box<dyn Perform>) -> Self {
        self.pending_actions.push(action);
        self
    }

    /// Pops the next action off of the stack of actions to complete.
    pub fn pop_next_action(&mut self) -> Option<Box<dyn Perform>> {
        self.pending_actions.pop()
    }

    /// For an action in a pipeline tha was popped, this will consume that action
    /// again and take result for that action.
    pub fn complete_action(&mut self, action: Box<dyn Perform>, action_result: ActionResult) {
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
