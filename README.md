# cloud-conveyor
A Self-Hosted, No Maintenance CI/CD Infrastructure for AWS. Pay entirely on usage.

* https://github.com/rusoto/rusoto
* https://docs.rs/slack/0.23.0/slack/
* https://docs.rs/regex/1.3.7/regex/
* https://github.com/chyh1990/yaml-rust
* https://github.com/awslabs/aws-lambda-rust-runtime

Applications Table
===============================
ORG / Name -> Hash and Range
Accounts
Approvals
Triggers
Prs
Stages


Deployments Table
===============================
Org+Name+Stage-Name -> Hash key
Sha -> Range Key
Is Deploying
Was Success
Bucket For Artifacts
Folder For Artifacts
Trigger
Caused By
Approval Status (Pending / Not Needed / Approved / Rejected / Unasked )