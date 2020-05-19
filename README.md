# Cloud Conveyor
Build and Deploy Service Pipelines with Chat Ops Anywhere.

## Motivation
Many CI / CD Providers have a pretty significant markup on build time compared to a cloud provider. And cloud provider solutions like CodePipeline are by there nature vendor specific and make it difficult to switch between cloud providers, make complex pipelines, or deploy to more than one cloud provider. 

While not there yet, the ultimate goal of this project is to solve all of those shortcomings and make a awesome developer-friendly system to build the software you want without vendor annoyances. 

## Current State
The current state of Cloud Conveyor is currently very much in development. The abstract pipeline and approval patterns are currently being fleshed out and work to begin to provide implementations for AWS, Github, and Slack are on the way.

Check out the `.conveyor.sample.yaml` file to see what it would be like to work with 
cloud conveyor. The build section is noticeably missing. The process of how best to implement that is not completed yet (See Below).

## Road Map
* Create Abstraction to Teardown Infrastructure
* Complete Abstraction for Build and Deploy
* Complete Pipeline State Machines
* Implement Abstractions for Github, Aws, and Slack
* Create Reference Deployment for Cloud Conveyor For Aws
* Support Build Secrets
* Create Standardized Build Specification File
* Complete CLI With Onboarding and other Helpful Commands.

## Contributing
Hope you like rust because basically everything about this project is written it it. If you are not familiar with the project, I would encourage you to check it out [here](https://www.rust-lang.org/). More instructions on working around the 
