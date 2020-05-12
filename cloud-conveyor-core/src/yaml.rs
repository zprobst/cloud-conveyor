//! Defines the method by which we load the application configuration
//! from a yaml file.
extern crate yaml_rust;
use std::fs::read_to_string;

use crate::{Account, Application, ApprovalGroup, Stage, Trigger};

struct AccountsSection {
    aws: Vec<Account>
}

struct ConfigFile {
    org: String,
    app: String,
    accounts: AccountsSection,
    
}

impl Into<Application> for ConfigFile {
    fn into(self) -> Application { todo!() }
}

pub fn load_app_from_yaml() -> Result<Application, ()> {
    // TODO:  Should this be customizable some how? If not, is this really the
    // best place for this value to be stored?
    let location = ".conveyor.sample.yaml";
    load_app_from_yaml_file(location)
}
