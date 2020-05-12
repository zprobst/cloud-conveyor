//! Defines the method by which we load the application configuration
//! from a yaml file.
extern crate serde;

use serde::{Deserialize, Serialize,};
use serde_yaml::{from_reader, to_string};

use std::io::prelude::*;
use std::fs::File;
use std::collections::HashMap;

use crate::{Account, Application, ApprovalGroup, Stage, Trigger};

/// Defines the yaml file defintion for an approval type.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApprovalDefinition {
    /// The type of approval that is supported. See ApprovalGroup for all
    /// types.
    r#type: String,

    /// The people on that approval pattern that are required.
    people: Vec<String>
}

impl Into<ApprovalGroup> for ApprovalDefinition {
    fn into(self) -> ApprovalGroup { 
        ApprovalGroup::Slack {
            people: self.people
        }
    }
}

/// Defines the yaml file definition for a stage.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StageDefinition {
    /// The name of the stage. eg. dev, stage, prod.
    pub name: String,

    /// The name of the approvers group for prod.
    pub approvers: Option<String>,

    /// The name of the account to deploy with.
    pub account: Option<String>
}

impl StageDefinition {
    fn into_stage(&self, approvers: &HashMap<String, ApprovalDefinition>, accounts: &Vec<Account>) -> Stage {
        let name = self.name.clone();
        let target = self.account.clone().unwrap_or(String::from("default"));
        let account = accounts.iter().find(|&acc| acc.is_named(target.as_str()));
        let approval_group = self.approvers
            .as_ref()
            .map(|app| &approvers[app])
            .map(|def| def.clone().into());

        Stage {
            name,
            approval_group,
            account: account.expect("The account was not found for the sage.").clone()
        }
    }
}


/// The root configuration file object. This is a represnetation of 
/// what the user has stored at a given version of their ".conveyor.yaml" 
/// file.
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigFile {
    /// The org of the config.
    pub org: String,
    /// The app of the config.
    pub app: String,
    /// The accounts of the config.
    pub accounts: Vec<Account>,
    /// The stages of the config.
    pub stages: Vec<StageDefinition>,
    /// The triggers of the config.
    pub triggers: Vec<Trigger>,
    /// The approvals of the config.
    pub approvals: HashMap<String, ApprovalDefinition>
}

impl Into<Application> for ConfigFile {
    /// Taken the Configuration File, computes an application instance.
    fn into(self) -> Application {
        let default_account_index = self.accounts
            .iter()
            .position(|acc| acc.is_candidate_for_default());
        let stages = self.stages
            .iter()
            .map(|s| s.into_stage(&self.approvals, &self.accounts))
            .collect();
        let approval_groups = self.approvals
            .iter()
            .map(|(_, val)| val.clone().into())
            .collect();

        Application {
            org: self.org,
            app: self.app,
            triggers: self.triggers,
            accounts: self.accounts,
            approval_groups,
            default_account_index,
            stages
        }
    }
}

/// Loads a configuration as a config file struct.
pub fn load_conf_from_yaml() -> Result<ConfigFile, std::io::Error> {
    let file = File::open(".conveyor.yaml")?;
    let yaml: ConfigFile = from_reader(file).expect("file is not valid yaml format");
    return Ok(yaml);
}

/// Loads a configuration as an application struct.
pub fn load_app_from_yaml() -> Result<Application, std::io::Error> {
    return load_conf_from_yaml().map(|conf| conf.into());
}

/// Creates a new file and saves it in the current directory for the config.
pub fn write_new_config_file(app: String, org: String) -> Result<(), std::io::Error> {
    let mut file = File::create(".conveyor.yaml")?;

    let config = ConfigFile {
        app,
        org,
        accounts: vec![Account{
            name: String::from("default"),
            id: 123456789,
            regions: vec!["us-east-1".to_owned()]
        }],
        approvals: HashMap::new(),
        triggers: vec![
            Trigger::Pr{deploy: true}, 
            Trigger::Merge{to: "master".to_owned(), from: None, stages: vec!["stage".to_owned()]},
            Trigger::Tag{pattern: "semver".to_owned(), stages: vec!["prod".to_owned()]}
        ],
        stages: vec![
            StageDefinition {
                name: "stage".to_owned(),
                approvers: None,
                account: None   
            },
            StageDefinition {
                name: "prod".to_owned(),
                approvers: None,
                account: None   
            }
        ]
    };

    let config_string = to_string(&config).unwrap();
    file.write_all(config_string.as_ref()).unwrap();
    Ok(())
}