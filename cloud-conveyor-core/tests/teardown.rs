use cloud_conveyor_core::pipelining::Teardown;
use cloud_conveyor_core::runtime::RuntimeContext;
use cloud_conveyor_core::teardown::{TeardownInfrastructure, TeardownPollError, TeardownStatus};
#[cfg(test)]
#[derive(Debug)]
struct TestImpl(
    Result<(), TeardownPollError>,
    Result<TeardownStatus, TeardownPollError>,
);

impl TeardownInfrastructure for TestImpl {
    fn start_teardown(
        &self,
        deploy: &Teardown,
        ctx: &RuntimeContext,
    ) -> Result<(), TeardownPollError> {
        self.0.to_owned()
    }

    fn check_teardown(
        &self,
        deploy: &Teardown,
        ctx: &RuntimeContext,
    ) -> Result<TeardownStatus, TeardownPollError> {
        self.1.to_owned()
    }
}

#[test]
fn teardown_errors_when_start_fails() {}
