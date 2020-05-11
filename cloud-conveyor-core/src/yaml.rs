//! Defines the method by which we load the application configuration
//! from a yaml file.
extern crate yaml_rust;
use std::fs::read_to_string;
use yaml_rust::YamlLoader;

use crate::{Account, Application, ApprovalGroup, Stage, Trigger};

// TODO: Get a better error  in load_app_from_yaml.
// TODO: Loader functions are going to need errors.
// TOOD: Actually handle errors.

fn load_approval_groups() -> Vec<ApprovalGroup> {
    Vec::new()
}

fn load_app_from_yaml_file(file: &str) -> Result<Application, ()> {
    // Load the document.
    let yaml = read_to_string(file).unwrap();
    let docs = YamlLoader::load_from_str(yaml.as_str()).unwrap();
    let doc = &docs[0];

    // Start to load trival information from the docs.
    let org = doc["org"].as_str().unwrap();
    let app = doc["app"].as_str().unwrap();


    Err(())
}

pub fn load_app_from_yaml() -> Result<Application, ()> {
    // TODO:  Should this be customizable some how? If not, is this really the
    // best place for this value to be stored?
    let location = "conveyor.yaml";
    load_app_from_yaml_file(location)
}
