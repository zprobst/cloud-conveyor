//! Provides the abstractions and core logic for the different implementations of the cloud conveyor runtime.
//!
//! When used in this project the term "runtime" defines a series of implementations that are specific to the cloud
//! provider in which cloud conveyor is running. Traits such as where to store artifacts, how to build the application,
//! and the like are defined here and provide the standard interface bindings that all of the runtime implementations
//! can provide.
//!

use crate::Application;
use std::fmt::Debug;

/// TODO
pub trait ProvideArtifact: Debug {
    /// TODO
    fn get_bucket(&self, app: &Application) -> String;
    /// TODO
    fn get_folder(&self, app: &Application, git_sha: &str) -> String;
}

/// TODO
pub trait Build: Debug {}

/// TODO
#[derive(Debug)]
pub struct RuntimeContext<'artifact, 'builder> {
    /// TODO
    pub artifact_provider: &'artifact mut dyn ProvideArtifact,
    /// TODO
    pub builder: &'builder mut dyn Build,
}

impl<'artifact, 'builder> RuntimeContext<'artifact, 'builder> {
    /// Builds a new runtime context that will be passed to different things in the runtime.
    pub fn new(
        artifact_provider: &'artifact mut dyn ProvideArtifact,
        builder: &'builder mut dyn Build,
    ) -> Self {
        Self {
            artifact_provider,
            builder,
        }
    }

    /// TODO
    pub fn load_application_from_repo(&self, _: &str) -> Option<&mut Application> {
        unimplemented!();
    }
}
