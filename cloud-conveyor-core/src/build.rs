//! Defines the runtime abstraction for build source and reporting successes and failures when doing so.
use crate::pipelining::Build;
use crate::runtime::RuntimeContext;
use crate::Application;

use async_trait::async_trait;
use failure::Error;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Determines the current status of a stack deployment. This information should signal
/// the state of the stack itself and not the result of performing the operation to check
/// the state. For instance, if an api call fails when trying to check the state, `Failed` should
/// not be returned. Instead the Result should be Err and the appropriate error information
/// should be provided by [BuildPollError](enum.BuildPollError.html).]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum BuildStatus {
    /// Indicates that the build was complete and the result was a success.
    Succeeded {
        /// Url that can be clicked on to view logs.
        logs: String,
    },
    /// Indicates that the build failed. This will result in a cancellation of the pipeline.
    Failed {
        /// Url that can be clicked on to view logs.
        logs: String,
        /// Additional error information
        error: Option<String>,
    },
    /// Indicates that the result is unknown currently and that this should be polled later.
    Pending,
}

/// Defines an error that occurrent when attempting to perform an operation on the
/// [BuildSource](trait.BuildSource.html) trait. This is meant to convey
/// that the operation was not a success - not that the deployment itself was a failure. That
/// information can be conveyed by returning Ok with a `Failed` value in [BuildStatus](enum.BuildStatus.html).
#[derive(Debug, Fail)]
pub enum BuildPollError {
    /// When credentials are an issue, either because they were considered invalid or because of
    /// they could not be obtained for some reason, this variant should be used.
    #[fail(display = "Failed to get credentials or the credentials were invalid.")]
    Credentials,
    /// When the cause does not fit any of the known patterns defined else where in the enum,
    /// this can be returned. It takes an additional string and passed that information as part
    /// of the error  context.
    #[fail(display = "Unknown build error occurred: {}", info)]
    Other {
        /// Additional error information.
        info: String,
    },
}

/// Defines a run time abstraction for using an external tool to build the source of different projects.
///
/// Like many of the abstractions in Cloud Conveyor, we use a polling mechanism to adapt the code
///  to different platforms. To that end, this code has two different methods. One to [start](#tymethod.start_build)
/// a [Build](../pipelining/struct.Build.html) and one to [check](#tymethod.check_build) that status of
/// a [Build](../pipelining/struct.Build.html).
///
/// The `BuildSource` trait works hard to be provider agnostic. To do so, the [BuildStatus](enum.BuildStatus.html)
/// and [BuildPollError](enum.BuildPollError.html) types are somewhat generic and may lose detail for  the given
/// implementation for the provider of the builder. Thus, you are encouraged to add additional log statements inside
/// of your implementation for `BuildSource`.
///
/// Since cloud conveyor performs operations external to the context of cloud conveyor itself, these methods should not
/// do much of the work of actually building the code. For example, if you are on AWS, use code build
/// to perform the build and poll the state of the build to see if it is done as opposed to building it locally
/// or creating an ec2 instance to do the build. That is an extreme example, but the idea is to use the tools of the
/// provider you are working with to do the work for you and have this implement the polling and reporting functionality only.
#[async_trait]
pub trait BuildSource: Debug + Sync + Send {
    /// Starts the build of the code given with [Build](../pipelining/struct.Build.html) data passed.
    /// Builds should be stage agnostic and are potentially re-used for more  than one [stage](../struct.Stage.html)
    /// deployment later in the pipeline. As such, you are not given access to the stage(s) the code is being built for.
    ///
    /// If an error occurs when triggering the build, use the appropriate variant of  [BuildPollError](enum.BuildPollError.html)
    async fn start_build(
        &self,
        build: &Build,
        ctx: &RuntimeContext,
        app: &Application,
    ) -> Result<(), BuildPollError>;

    /// Polls the state of the build given the build data passed.
    /// If that final result of the build is not known use [BuildStatus::Pending](enum.BuildStatus.html#variant.Pending).
    /// If the build was completed completed successfully, or errors in an a way that does not effect the pipeline from
    /// continuing, regardless of the state of the pipeline (notice we give you no way to see that) you may use
    /// [BuildStatus::Complete](enum.BuildStatus.html#variant.Complete). If there was an error during the build that
    /// does not allow the continuation of the pipeline, then return [BuildStatus::Failed](enum.BuildStatus.html#variant.Failed)
    ///
    /// If an error occurs when polling the state of the build, use the appropriate variant of  [BuildPollError](enum.BuildPollError.html)
    async fn check_build(
        &self,
        build: &Build,
        ctx: &RuntimeContext,
        app: &Application,
    ) -> Result<BuildStatus, BuildPollError>;
}

/// Defines an abstraction for using an external tool to provide locations to store and retrieve artifacts from.
///
/// Unlike many of the abstractions in Cloud Conveyor, we do not use a polling mechanism to adapt the code
///  to different platforms. This is also not used much internally to the code base and instead passed to things
/// like [BuildSource](trait.BuildSource.html) and [DeployInfrastructure](../deploy/trait.DeployInfrastructure.html)
/// indirectly through the [RuntimeContext](../runtime/struct.RuntimeContext.html).
///
/// The `ProvideArtifactLocation` trait works hard to be provider agnostic. To do so, the Result type is fairly generic
/// and is a [failure::Error](../../failure/struct.Error.html) and has Ok type of String to represent the file paths.
#[async_trait]
pub trait ProvideArtifactLocation: Debug + Sync + Send {
    /// Gets the name of a storage bucket or location without a path to store the assets. This trait instance
    /// will be passed to the activated implementations of [BuildSource](trait.BuildSource.html) and
    /// [DeployInfrastructure](../deploy/trait.DeployInfrastructure.html) when they are invoked so they should
    /// understand and be able to interpret this value or need not to.
    async fn get_bucket(&self, app: &Application) -> Result<String, Error>;
    /// Gets the path to a folder in  a storage bucket from above. This trait instance
    /// will be passed to the activated implementations of [BuildSource](trait.BuildSource.html) and
    /// [DeployInfrastructure](../deploy/trait.DeployInfrastructure.html) when they are invoked so they should
    /// understand and be able to interpret this value or need not to.
    async fn get_folder(&self, app: &Application, git_sha: &str) -> Result<String, Error>;
}
