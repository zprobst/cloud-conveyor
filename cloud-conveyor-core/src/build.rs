//! Defines the runtime abstraction for build source and reporting successes and failures when doing so.
use crate::pipelining::Build;
use crate::runtime::RuntimeContext;
use crate::Application;

use failure::Error;
use std::fmt::Debug;

/// Determines the current status of a stack deployment. This information should signal
/// the state of the stack itself and not the result of performing the operation to check
/// the state. For instance, if an api call fails when trying to check the state, `Failed` should
/// not be returned. Instead the Result should be Err and the appropriate error information
/// should be provided by [BuildPollError](enum.BuildPollError.html).]
#[derive(Debug)]
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
        /// Addtional error information.
        info: String,
    },
}

/// TODO
pub trait BuildSource: Debug {
    /// TODO
    fn start_build(&self, build: &Build, ctx: &RuntimeContext) -> Result<(), BuildPollError>;

    /// TODO
    fn check_deployment(
        &self,
        build: &Build,
        ctx: &RuntimeContext,
    ) -> Result<BuildStatus, BuildPollError>;
}

/// TODO
pub trait ProvideArtifact: Debug {
    /// TODO
    fn get_bucket(&self, app: &Application) -> Result<String, Error>;
    /// TODO
    fn get_folder(&self, app: &Application, git_sha: &str) -> Result<String, Error>;
}
