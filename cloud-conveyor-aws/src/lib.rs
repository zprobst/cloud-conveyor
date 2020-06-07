use cloud_conveyor_core::{
    build::{BuildPollError, BuildSource, BuildStatus, ProvideArtifactLocation},
    deploy::{DeployInfrastructure, DeployPollError, DeployStatus},
    pipelining::{Build, Deploy, Teardown},
    runtime::RuntimeContext,
    teardown::{TeardownInfrastructure, TeardownPollError, TeardownStatus},
    Application,
};

use async_trait::async_trait;
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

#[async_trait]
impl TeardownInfrastructure for Aws {
    async fn start_teardown(
        &self,
        _: &Teardown,
        _: &RuntimeContext,
    ) -> Result<(), TeardownPollError> {
        todo!()
    }
    async fn check_teardown(
        &self,
        _: &Teardown,
        _: &RuntimeContext,
    ) -> Result<TeardownStatus, TeardownPollError> {
        todo!()
    }
}

#[async_trait]
impl DeployInfrastructure for Aws {
    async fn start_deployment(
        &self,
        _: &Deploy,
        _: &RuntimeContext,
    ) -> Result<(), DeployPollError> {
        todo!()
    }
    async fn check_deployment(
        &self,
        _: &Deploy,
        _: &RuntimeContext,
    ) -> Result<DeployStatus, DeployPollError> {
        todo!()
    }
}

#[async_trait]
impl BuildSource for Aws {
    async fn start_build(&self, _: &Build, _: &RuntimeContext) -> Result<(), BuildPollError> {
        todo!()
    }
    async fn check_build(
        &self,
        _: &Build,
        _: &RuntimeContext,
    ) -> Result<BuildStatus, BuildPollError> {
        todo!()
    }
}

#[async_trait]
impl ProvideArtifactLocation for Aws {
    async fn get_bucket(&self, _: &Application) -> Result<String, Error> {
        todo!()
    }
    async fn get_folder(&self, _: &Application, _: &str) -> Result<String, Error> {
        todo!()
    }
}
