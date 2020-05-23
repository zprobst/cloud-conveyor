use crate::pipelining::{ActionResult, Perform, Pipeline};
use crate::runtime::RuntimeContext;

use failure::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct StateMachine {
    pipeline: Pipeline,
    current_action: Option<Box<dyn Perform>>,
}

impl StateMachine {
    pub fn new(pipeline: Pipeline) -> Self {
        Self {
            pipeline,
            current_action: None,
        }
    }

    fn tick_machine_state(&mut self, context: &RuntimeContext) -> Result<bool, Error> {
        // Get the current action and see if it is done.
        let mut start_action = match self.current_action {
            Some(_) => false,
            None => true,
        };

        if !start_action {
            let mut action = self
                .current_action
                .as_mut()
                .expect("Action does not exist despite just checking its value");
            let is_done = action.is_done(context)?;
            if is_done {
                start_action = true;
                // We need to know the state of the job's result.
                let result = action.get_result(context);

                let cancel = match result {
                    ActionResult::Success => false,
                    ActionResult::FailedAllow => false,
                    ActionResult::Failed => true,
                    ActionResult::Canceled => true,
                };

                if cancel {
                    self.pipeline.cancel()
                }

                // If there is new work, we will push these items
                if let Some(actions) = action.get_new_work(context) {
                    for action in actions {
                        self.pipeline.add_immediate_action(action);
                    }
                }
            }
        }

        if start_action {
            // Pop a new action.
            // start it.
            // poll its state right away (recursively)
        }

        Ok(false)
    }
}
