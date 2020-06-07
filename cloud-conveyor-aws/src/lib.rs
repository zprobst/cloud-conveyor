use cloud_conveyor_core::{
    build::{BuildPollError, BuildSource, BuildStatus, ProvideArtifactLocation},
    deploy::{DeployInfrastructure, DeployPollError, DeployStatus},
    pipelining::{Build, Deploy, Teardown},
    runtime::RuntimeContext,
    teardown::{TeardownInfrastructure, TeardownPollError, TeardownStatus},
    Application,
};

use derivative::Derivative;
use failure::Error;
use rusoto_cloudformation::{CloudFormation, CloudFormationClient};
use rusoto_codebuild::{CodeBuild, CodeBuildClient};
use rusoto_core::Region;

#[derive(Derivative)]
#[derivative(Debug)]
struct Aws {
    #[derivative(Debug = "ignore")]
    cloudformation_client: CloudFormationClient,
    #[derivative(Debug = "ignore")]
    codebuild_client: CodeBuildClient,
    region: Region,
}

impl TeardownInfrastructure for Aws {
    fn start_teardown(&self, _: &Teardown, _: &RuntimeContext) -> Result<(), TeardownPollError> {
        todo!()
    }
    fn check_teardown(
        &self,
        _: &Teardown,
        _: &RuntimeContext,
    ) -> Result<TeardownStatus, TeardownPollError> {
        todo!()
    }
}

impl DeployInfrastructure for Aws {
    fn start_deployment(&self, _: &Deploy, _: &RuntimeContext) -> Result<(), DeployPollError> {
        todo!()
    }
    fn check_deployment(
        &self,
        _: &Deploy,
        _: &RuntimeContext,
    ) -> Result<DeployStatus, DeployPollError> {
        todo!()
    }
}

impl BuildSource for Aws {
    fn start_build(&self, _: &Build, _: &RuntimeContext) -> Result<(), BuildPollError> {
        todo!()
    }
    fn check_build(&self, _: &Build, _: &RuntimeContext) -> Result<BuildStatus, BuildPollError> {
        todo!()
    }
}

impl ProvideArtifactLocation for Aws {
    fn get_bucket(&self, _: &Application) -> Result<String, Error> {
        todo!()
    }
    fn get_folder(&self, _: &Application, _: &str) -> Result<String, Error> {
        todo!()
    }
}
