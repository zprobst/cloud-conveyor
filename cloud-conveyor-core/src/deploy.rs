//! Defines the runtime abstraction for deploying infrastructure and reporting successes and failures when doing so.
use crate::pipelining::Deploy;
use crate::runtime::RuntimeContext;
use crate::Application;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Defines an error that occurrent when attempting to perform an operation on the
///  [DeployInfrastructure](trait.DeployInfrastructure.html) trait. This is meant to convey
/// that the operation was not a success - not that the deployment itself was a failure. That
/// information can be conveyed by returning Ok with a `Failed` value in [DeployStatus](enum.DeployStatus.html).
#[derive(Debug, Fail)]
pub enum DeployPollError {
    /// When credentials are an issue, either because they were considered invalid or because of
    /// they could not be obtained for some reason, the  Credentials variant should be used.
    #[fail(display = "Failed to get credentials or the credentials were invalid.")]
    Credentials,

    /// When the cause does not fit any of the known patterns defined else where in the enum,
    /// this can be returned. It takes an additional string and passed that information as part
    /// of the error  context.
    #[fail(display = "Unknown deployment error occurred: {}", info)]
    Other {
        /// Additional information to pass back.
        info: String,
    },
}

/// Determines the current status of a stack deployment. This information should signal
/// the state of the stack itself and not the result of performing the operation to check
/// the state. For instance, if an api call fails when trying to check the state, `Failed` should
/// not be returned. Instead the Result should be Err and the appropriate error information
/// should be provided by [DeployPollError](enum.DeployPollError.html).]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum DeployStatus {
    /// Indicates that the stack update was a complete and the result was a success.
    Complete,
    /// Indicates that the deployment failed. This will result in a cancellation of the pipeline,
    Failed,
    /// Indicates that the result is unknown currently and that this should be polled later to be sure.
    Pending,
}

/// The `DeployInfrastructure` is at the heart of cloud conveyor and is used to, you guessed, it
/// deploy infrastructure. To that end, it like many things in Cloud Conveyor use a polling mechanism
/// controlled by the internal state machine that the cloud conveyor core library implements.
///
/// The `DeployInfrastructure` trait works hard to be provider agnostic. To do so, the [DeployStatus](enum.DeployStatus.html)
/// and [DeployPollError](enum.DeployPollError.html) types are somewhat generic and may lose detail for  the given
/// implementation for the provider of the infrastructure. Thus, you are encouraged to add additional log statements inside
/// of your implementation for `DeployInfrastructure`.
///
/// Since cloud conveyor performs operations external to the context of cloud conveyor itself, these methods should not
/// do much of the work of actually deploying the infrastructure. For example, if you are deploying to AWS, use cloud formation
/// to deploy the infrastructure and poll the state of the stack to see if it is done as opposed to making api calls it create / update / delete
/// infrastructure for stacks. That is an extreme example, but the idea is to use the tools of the provider you are deploying to do the work
/// for you and have this implement the polling and reporting functionality.
#[async_trait]
pub trait DeployInfrastructure: Debug + Sync + Send {
    /// Starts the deployment of the infrastructure given the deployment data passed. Keep in mind that we do not,
    /// determine whether or not a stack exists ahead of time. To start a deployment, you must check if a stack exists
    /// and if not create the stack if required by the cloud provider. If the stack is there, you should update it.
    ///
    /// If an error occurs when triggering the deployment, use the appropriate variant of  [DeployPollError](enum.DeployPollError.html)
    async fn start_deployment(
        &self,
        deploy: &Deploy,
        ctx: &RuntimeContext,
        app: &Application,
    ) -> Result<(), DeployPollError>;

    /// Polls the state of the deployment of the infrastructure given the deployment data passed.
    /// If that final result of the deployment is not known use [DeployStatus::Pending](enum.DeployStatus.html#variant.Pending).
    /// If the stack  create or update was completed successfully, or errors in an a way that does not effect the pipeline from
    /// continuing, regardless of the state of the pipeline (notice we give you no way to see that) you may use
    /// [DeployStatus::Complete](enum.DeployStatus.html#variant.Complete). If there was an error during the deployment that
    /// does not allow the continuation of the pipeline, then return [DeployStatus::Failed](enum.DeployStatus.html#variant.Failed)
    ///
    /// If an error occurs when polling the state of the deployment, use the appropriate variant of  [DeployPollError](enum.DeployPollError.html)
    async fn check_deployment(
        &self,
        deploy: &Deploy,
        ctx: &RuntimeContext,
        app: &Application,
    ) -> Result<DeployStatus, DeployPollError>;
}
