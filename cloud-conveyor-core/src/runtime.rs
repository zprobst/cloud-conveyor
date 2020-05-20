//! Provides the abstractions and core logic for the different implementations of the cloud conveyor runtime.
//!
//! When used in this project the term "runtime" defines a series of implementations that are specific to the cloud
//! provider in which cloud conveyor is running. Traits such as where to store artifacts, how to build the application,
//! and the like are defined here and provide the standard interface bindings that all of the runtime implementations
//! can provide.

use crate::build::{BuildSource, ProvideArtifactLocation};
use crate::deploy::DeployInfrastructure;
use crate::teardown::TeardownInfrastructure;
use crate::Application;
use std::fmt::Debug;

/// TODO
#[derive(Debug)]
pub struct RuntimeContext {
    /// TODO
    pub artifact_provider: Box<dyn ProvideArtifactLocation>,
    /// TODO
    pub builder: Box<dyn BuildSource>,
    /// TODO
    pub infrastructure: Box<dyn DeployInfrastructure>,
    /// TODO
    pub teardown: Box<dyn TeardownInfrastructure>,
}

impl RuntimeContext {
    /// TODO
    pub fn load_application_from_repo(&self, _: &str) -> Option<&mut Application> {
        unimplemented!();
    }
}
