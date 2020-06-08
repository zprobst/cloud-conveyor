//! Defines the runtime abstraction for tearing down infrastructure and reporting successes and failures when doing so.
use crate::pipelining::Teardown;
use crate::runtime::RuntimeContext;
use crate::Application;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Defines an error that occurrent when attempting to perform an operation on the
///  [TeardownInfrastructure](trait.TeardownInfrastructure.html) trait. This is meant to convey
/// that the operation was not a success - not that the deployment itself was a failure. That
/// information can be conveyed by returning Ok with a `Failed` value in [DeployStatus](enum.DeployStatus.html).
#[derive(Debug, Clone, Fail)]
pub enum TeardownPollError {
    /// When credentials are an issue, either because they were considered invalid or because of
    /// they could not be obtained for some reason, the  Credentials variant should be used.
    #[fail(display = "Failed to get credentials or the credentials were invalid.")]
    Credentials,

    /// When a stack cannot be deleted entirely then this should be used.
    #[fail(display = "When a stack cannot be deleted entirely and should be.")]
    CannotDelete,

    /// When the cause does not fit any of the known patterns defined else where in the enum,
    /// this can be returned. It takes an additional string and passed that information as part
    /// of the error  context.
    #[fail(display = "Unknown deployment error occurred: {}", info)]
    Other {
        /// Additional information to pass back.
        info: String,
    },
}

/// Determines the current status of a stack teardown. This information should signal
/// the state of the stack itself and not the result of performing the operation to check
/// the state. For instance, if an api call fails when trying to check the state, `Failed` should
/// not be returned. Instead the Result should be Err and the appropriate error information
/// should be provided by [TeardownPollError](enum.TeardownPollError.html).]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum TeardownStatus {
    /// Indicates that the stack update was a complete and the result was a success.
    Complete,
    /// Indicates that the teardown failed. This will result in a cancellation of the pipeline,
    Failed,
    /// Indicates that the result is unknown currently and that this should be polled later to be sure.
    Pending,
}

/// The `TeardownInfrastructure` is at the heart of cloud conveyor and is used to, you guessed, it
/// teardown infrastructure. To that end, it like many things in Cloud Conveyor use a polling mechanism
/// controlled by the internal state machine that the cloud conveyor core library implements.
///
/// The `TeardownInfrastructure` trait works hard to be provider agnostic. To do so, the [TeardownStatus](enum.TeardownStatus.html)
/// and [TeardownPollError](enum.TeardownPollError.html) types are somewhat generic and may lose detail for  the given
/// implementation for the provider of the infrastructure. Thus, you are encouraged to add additional log statements inside
/// of your implementation for `TeardownInfrastructure`.
///
/// Since cloud conveyor performs operations external to the context of cloud conveyor itself, these methods should not
/// do much of the work of actually destroying the infrastructure. For example, if you are deploying to AWS, use cloud formation
/// to delete the infrastructure and poll the state of the stack to see if it is done as opposed to making api calls it delete
/// infrastructure. That is an extreme example, but the idea is to use the tools of the provider you are deploying to do the work
/// for you and have this implement the polling and reporting functionality.
#[async_trait]
pub trait TeardownInfrastructure: Debug + Sync + Send {
    /// Starts the teardown of the infrastructure given the data passed. Keep in mind that we do not,
    /// it is considered an invariant that we call a teardown on a stage that was never deployed.
    ///
    /// If an error occurs when triggering the teardown, use the appropriate variant of  [TeardownPollError](enum.TeardownPollError.html)
    async fn start_teardown(
        &self,
        deploy: &Teardown,
        ctx: &RuntimeContext,
        app: &Application,
    ) -> Result<(), TeardownPollError>;

    /// Polls the state of the teardown of the infrastructure given the data passed.
    /// If that final result of the teardown is not known use [TeardownStatus::Pending](enum.TeardownStatus.html#variant.Pending).
    /// If the stack  create or update was completed successfully, or errors in an a way that does not effect the pipeline from
    /// continuing, regardless of the state of the pipeline (notice we give you no way to see that) you may use
    /// [TeardownStatus::Complete](enum.TeardownStatus.html#variant.Complete). If there was an error during the teardown that
    /// does not allow the continuation of the pipeline, then return [TeardownStatus::Failed](enum.TeardownStatus.html#variant.Failed)
    ///
    /// If an error occurs when polling the state of the teardown, use the appropriate variant of  [TeardownStatus](enum.DeployPollError.html)
    async fn check_teardown(
        &self,
        deploy: &Teardown,
        ctx: &RuntimeContext,
        app: &Application,
    ) -> Result<TeardownStatus, TeardownPollError>;
}
