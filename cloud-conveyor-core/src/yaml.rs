//! Defines the method by which we load the application configuration 
//! from a yaml file.
extern crate yaml_rust;
use yaml_rust::YamlLoader;

// TODO: Get a better error  in load_app_from_yaml.
// TODO: Loader functions are going to need errors.

use crate::{Account, Application, ApprovalGroup, Stage, Trigger};

fn load_approval_groups() -> Vec<ApprovalGroup> {
    Vec::new()
}

pub fn load_app_from_yaml() -> Result<Application<'static>, ()> {
    // TODO:  Should this be customizable some how? If not, is this really the 
    // best place for this value to be stored? 
    let location = "conveyor.yaml";
    Err(())
}