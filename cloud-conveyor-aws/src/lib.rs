//! Defines the implementations of building and deploying on AWS for cloud conveyor.
#![warn(
    missing_docs,
    rust_2018_idioms,
    missing_debug_implementations,
    intra_doc_link_resolution_failure
)]

use cloud_conveyor_core::{
    build::{BuildPollError, BuildSource, BuildStatus, ProvideArtifactLocation},
    deploy::{DeployInfrastructure, DeployPollError, DeployStatus},
    pipelining::{Build, Deploy, Teardown},
    runtime::RuntimeContext,
    teardown::{TeardownInfrastructure, TeardownPollError, TeardownStatus},
    Application, Stage,
};

use async_trait::async_trait;
use failure::Error;
use rusoto_cloudformation::{
    CloudFormation, CloudFormationClient, DeleteStackInstancesInput, DeleteStackSetInput,
    ListStackInstancesInput,
};
//use rusoto_codebuild::{CodeBuild, CodeBuildClient};
use rusoto_core::{request::HttpClient, Region};
use rusoto_credential::ProvideAwsCredentials;
use rusoto_sts::{StsAssumeRoleSessionCredentialsProvider, StsClient};

use std::collections::HashMap;
use std::fmt::Debug;

// TODO: For now, we are always swallowing if there is more that one error and only returning
// the first one, maybe we should change the error API to return a list of errors.

/// Builds a copy of the `Aws` Struct such that the it can potentially assume multiple roles
/// to different accounts. Here is an example usage.todo!
///
/// ```rust
/// use cloud_conveyor_aws::Aws;
/// use rusoto_credential::EnvironmentProvider;
/// use rusoto_core::Region;
///
/// let aws = Aws::build()
///     .bucket("my-bucket-name".to_owned())
///     .add_account_role(
///         123456789012,
///         "arn:aws:iam::123456789012:role/CloudConveyor".to_owned(),
///     )
///     .add_external_id(
///         123456789012,
///         "some-super-secret-value".to_owned(),
///     )
///     .add_account_role(
///         210987654321,
///         "arn:aws:iam::210987654321:role/CloudConveyor".to_owned(),
///     )
///     .region(Region::UsEast1)
///     .credentials(EnvironmentProvider::default())
///     .finish();
/// ```
#[derive(Debug)]
pub struct AwsBuilder<P>
where
    P: ProvideAwsCredentials + Clone + Send + Sync + Debug + 'static,
{
    bucket: Option<String>,
    account_role_map: Option<HashMap<usize, String>>,
    external_id_map: Option<HashMap<usize, String>>,
    credentials: Option<P>,
    region: Option<Region>,
}

impl<P> AwsBuilder<P>
where
    P: ProvideAwsCredentials + Clone + Send + Sync + Debug + 'static,
{
    /// Sets the name of the bucket that is expected in which the assets should be stored.
    pub fn bucket(mut self, bucket: String) -> Self {
        self.bucket = Some(bucket);
        self
    }

    /// Sets the name of the region that is expected in which sts
    /// assumption should be set.
    pub fn region(mut self, region: Region) -> Self {
        self.region = Some(region);
        self
    }

    /// Adds an role for a specified account.
    pub fn add_account_role(mut self, account: usize, role: String) -> Self {
        if self.account_role_map.is_none() {
            self.account_role_map = Some(HashMap::new());
        }
        self.account_role_map
            .as_mut()
            .expect("account_role_map is empty while just checking its existence")
            .insert(account, role);
        self
    }
    /// Adds an role for a specified account. This is optional for any given account depending
    /// on the requirements of the account and role that is being assumed.
    pub fn add_external_id(mut self, account: usize, external_id: String) -> Self {
        if self.external_id_map.is_none() {
            self.external_id_map = Some(HashMap::new());
        }
        self.external_id_map
            .as_mut()
            .expect("external_id_map is empty while just checking its existence")
            .insert(account, external_id);
        self
    }

    /// Sets the credentials to use to perform aws actions with. This roles is used for the given account
    /// for all actions unless an assume role is provided. If an assumable role is provided, it will use that
    /// instead of the credentials.
    pub fn credentials(mut self, credentials: P) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Finishes the build of the `Aws` struct. Panics if  no s3 bucket is specified or
    /// if no credentials are specified.
    pub fn finish(self) -> Aws<P> {
        Aws {
            account_role_map: self.account_role_map.unwrap_or_default(),
            external_id_map: self.external_id_map.unwrap_or_default(),
            bucket_name: self
                .bucket
                .expect("Did not set a bucket name during build."),
            credentials: self
                .credentials
                .expect("Did not set a credential provider during the build."),
            region: self.region.expect("Did not set a region to operate on"),
        }
    }
}

/// The AWS struct is responsible for performing build and infra operations
/// inside of aws accounts. Since the execution environment is unknown,
/// several things are required to be used ito build an instance of the `Aws`
/// struct. That is handled here with the [build](#method.build) method
/// as well as the [AwsBuilder](struct.AwsBuilder.html) struct.
///
/// While the `Aws` struct defines `DeployInfrastructure`, `TeardownInfrastructure`,
/// `BuildSource` and `ProvideArtifactLocation` implementations, this struct works
/// hard to not rely on the implementation of the other traits in this struct. That means,
/// for instance, you do not need to use aws to build and can still use aws to deploy so
/// long as certain invariants are met per-implementation. That is documented in the section
/// for each trait implementation below.
///
/// Additionally, the discussion for each trait will outline the information about what billing
/// will be incurred for that implementation. That may not be an exclusive list of charges.
///
/// ## TeardownInfrastructure
///  Briefly, the `TeardownInfrastructure` trait is responsible for removing stacks for environments
///  that are no longer in use. `CloudFormation` comes at no additional cost so not information should
///  not see any additional billing cost for the usage of this aspect of the `Aws` struct.
///  
///  ### TeardownInfrastructure Invariants
///  When given a particular stage of an application to remove, the expected stack name is in the form of
/// `{app-org}-{app-name]-{stage-name}`. This means that if `DeployInfrastructure` trait is not used from this
/// struct, your implementation  must conform to this specification. Somewhat implicitly, it must use cloudformation
///  and have a delete-able stack by the aforementioned name.
///
/// ## DeployInfrastructure
///  Briefly, the `DeployInfrastructure` trait is responsible for creating / updating stacks for environments
///  that are no longer in use. `CloudFormation` comes at no additional cost so not information should
///  not see any additional billing cost for the usage of this aspect of the `Aws` struct. Of course, billing
/// will be incurred for the resources in the trait.
///
///  ### DeployInfrastructure Invariants
/// In order to use the `DeployInfrastructure` implementation, there are a few invariants that are required
/// to be held. First, the converse of what is mentioned in the `TeardownInfrastructure` trait invariants section
/// needs to be held. That is, the name of the stack to be removed from `TeardownInfrastructure` should follow the
/// same pattern if your implementation differs.
///
/// Secondly, the artifacts must be stored in s3 in the location provided by the `RuntimeContext` implementation of
/// `ProvideArtifactLocation`.  Additionally, one of the artifacts provided by the build job _must_ be a file called
/// `template.yaml` that is expected to be a valid cloud formation template. Depending on the exact implementation
/// you are going for, this could be a constraint of the `ProvideArtifactLocation` implementation, the `BuildSource`
/// implementation, or both.
///
/// ## BuildSource
///  Briefly, the `BuildSource` trait is responsible for building your application and storing the artifacts in the location
/// specified by the `ProvideArtifactLocation` trait implementation on `RuntimeContext`.
///
/// ### BuildSource Invariants
/// Currently there are no Invariants not mentioned by DeployInfrastructure
///
/// ## ProvideArtifactLocation
///  Briefly, the `ProvideArtifactLocation` trait is specifying the location can be stored.
///
/// ### ProvideArtifactLocation Invariants
/// Currently there are no Invariants not mentioned by DeployInfrastructure.
///
#[derive(Debug)]
pub struct Aws<P>
where
    P: ProvideAwsCredentials + Clone + Send + Sync + Debug + 'static,
{
    bucket_name: String,
    region: Region,
    account_role_map: HashMap<usize, String>,
    external_id_map: HashMap<usize, String>,
    credentials: P,
}

impl<P> Aws<P>
where
    P: ProvideAwsCredentials + Clone + Send + Sync + Debug + 'static,
{
    /// Begins the build process of the `Aws` struct. Several things are required when
    /// building an environment.  Here is an example of building an complex multi-account
    /// system.
    ///
    /// ```rust
    /// use cloud_conveyor_aws::Aws;
    /// use rusoto_credential::EnvironmentProvider;
    /// use rusoto_core::Region;
    ///
    /// let aws = Aws::build()
    ///     .bucket("my-bucket-name".to_owned())
    ///     .add_account_role(
    ///         123456789012,
    ///         "arn:aws:iam::123456789012:role/CloudConveyor".to_owned(),
    ///     )
    ///     .add_external_id(
    ///         123456789012,
    ///         "some-super-secret-value".to_owned(),
    ///     )
    ///     .add_account_role(
    ///         210987654321,
    ///         "arn:aws:iam::210987654321:role/CloudConveyor".to_owned(),
    ///     )
    ///     .region(Region::UsEast1)
    ///     .credentials(EnvironmentProvider::default())
    ///     .finish();
    /// ```
    pub fn build() -> AwsBuilder<P>
    where
        P: ProvideAwsCredentials,
    {
        AwsBuilder {
            bucket: None,
            account_role_map: None,
            external_id_map: None,
            credentials: None,
            region: None,
        }
    }

    fn stack_name(&self, app: &Application, stage: &Stage) -> String {
        format!("{}-{}-{}", app.org, app.app, stage.name)
    }

    fn find_credentials(
        &self,
        account_no: &usize,
    ) -> Result<Option<StsAssumeRoleSessionCredentialsProvider>, Error> {
        if let Some(role) = self.account_role_map.get(account_no) {
            let external_id = self.account_role_map.get(account_no).cloned();
            Ok(Some(StsAssumeRoleSessionCredentialsProvider::new(
                StsClient::new_with(
                    HttpClient::new()?,
                    self.credentials.clone(),
                    self.region.clone(),
                ),
                role.clone(),
                format!("acc-{}", account_no),
                external_id,
                None,
                None,
                None,
            )))
        } else {
            Ok(None)
        }
    }

    fn cfn_client(&self, account_no: &usize) -> Result<CloudFormationClient, Error> {
        let http_client = HttpClient::new().map_err(|_| TeardownPollError::Other {
            info: "Http Client Failed to Create".to_owned(),
        })?;
        let credentials = self
            .find_credentials(account_no)
            .map_err(|_| TeardownPollError::Credentials)?
            .ok_or_else(|| TeardownPollError::Credentials)?;
        Ok(CloudFormationClient::new_with(
            http_client,
            credentials,
            self.region.clone(),
        ))
    }
}

#[async_trait]
impl<P> TeardownInfrastructure for Aws<P>
where
    P: ProvideAwsCredentials + Clone + Send + Sync + Debug + 'static,
{
    async fn start_teardown(
        &self,
        job: &Teardown,
        _: &RuntimeContext,
        app: &Application,
    ) -> Result<(), TeardownPollError> {
        let client = self
            .cfn_client(&job.stage.account.id)
            .map_err(|_| TeardownPollError::Credentials)?;

        // In-order to delete the stack-set, we need to delete all stack instances it has.
        // I.e We must remove the stack in all regions before we can continue. So we will
        // start by deleting all regions.
        let result = client
            .delete_stack_instances(DeleteStackInstancesInput {
                accounts: Some(vec![job.stage.account.id.to_string()]),
                regions: job.stage.account.regions.clone(),
                retain_stacks: false,
                stack_set_name: self.stack_name(app, &job.stage),
                ..Default::default()
            })
            .await
            .map_err(|e| TeardownPollError::Other {
                info: e.to_string(),
            })?;

        Ok(())
    }
    async fn check_teardown(
        &self,
        job: &Teardown,
        _: &RuntimeContext,
        app: &Application,
    ) -> Result<TeardownStatus, TeardownPollError> {
        let client = self
            .cfn_client(&job.stage.account.id)
            .map_err(|_| TeardownPollError::Credentials)?;

        let result = client
            .list_stack_instances(ListStackInstancesInput {
                max_results: Some(job.stage.account.regions.len() as i64),
                stack_set_name: self.stack_name(app, &job.stage),
                ..Default::default()
            })
            .await
            .map_err(|e| TeardownPollError::Other {
                info: e.to_string(),
            })?
            .summaries
            .unwrap();

        let any_errors = result.iter().any(|s| match s.status.as_ref() {
            Some(status) => match status.as_str() {
                "INOPERABLE" => true,
                _ => false,
            },
            None => false,
        });

        if result.len() == 0 {
            client
                .delete_stack_set(DeleteStackSetInput {
                    stack_set_name: self.stack_name(app, &job.stage),
                })
                .await
                .map_err(|e| TeardownPollError::Other {
                    info: e.to_string(),
                })?;
            Ok(TeardownStatus::Complete)
        } else {
            if any_errors {
                Ok(TeardownStatus::Failed)
            } else {
                Ok(TeardownStatus::Pending)
            }
        }
    }
}

#[async_trait]
impl<P> DeployInfrastructure for Aws<P>
where
    P: ProvideAwsCredentials + Clone + Send + Sync + Debug + 'static,
{
    async fn start_deployment(
        &self,
        _: &Deploy,
        _: &RuntimeContext,
        _: &Application,
    ) -> Result<(), DeployPollError> {
        todo!()
    }
    async fn check_deployment(
        &self,
        _: &Deploy,
        _: &RuntimeContext,
        _: &Application,
    ) -> Result<DeployStatus, DeployPollError> {
        todo!()
    }
}

#[async_trait]
impl<P> BuildSource for Aws<P>
where
    P: ProvideAwsCredentials + Clone + Send + Sync + Debug + 'static,
{
    async fn start_build(
        &self,
        _: &Build,
        _: &RuntimeContext,
        _: &Application,
    ) -> Result<(), BuildPollError> {
        todo!()
    }
    async fn check_build(
        &self,
        _: &Build,
        _: &RuntimeContext,
        _: &Application,
    ) -> Result<BuildStatus, BuildPollError> {
        todo!()
    }
}

/// We define a way of defining locations where we store artifacts in s3. We are given
/// a s3 bucket name that we will assume is the same for each account. This limitation
/// is a result of us only getting to know the application, and not the stage(and therefore account)
/// that the thing is getting built into.
#[async_trait]
impl<P> ProvideArtifactLocation for Aws<P>
where
    P: ProvideAwsCredentials + Clone + Send + Sync + Debug + 'static,
{
    async fn get_bucket(&self, _: &Application) -> Result<String, Error> {
        Ok(self.bucket_name.clone())
    }
    async fn get_folder(&self, app: &Application, git_ref: &str) -> Result<String, Error> {
        Ok(format!("{}/{}", app.full_name(), git_ref))
    }
}
