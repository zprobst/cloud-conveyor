//! Provides the abstractions and core logic for the different implementations of the cloud conveyor runtime.
//!
//! When used in this project the term "runtime" defines a series of implementations that are specific to the cloud
//! provider in which cloud conveyor is running. Traits such as where to store artifacts, how to build the application,
//! and the like are defined here and provide the standard interface bindings that all of the runtime implementations
//! can provide.
//!

use crate::Application;

pub trait ArtifactProvider {
    fn get_bucket(&self, app: &Application) -> String;
    fn get_folder(&self, app: &Application, git_sha: &str) -> String;
}

pub trait Builder {}
