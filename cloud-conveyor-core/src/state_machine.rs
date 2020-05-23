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
        let mut result = Self {
            pipeline,
            current_action: None,
        };
        result.current_action = result.pipeline.pop_next_action();
        result
    }

    /// Performs one cycle of the state machine by polling the current action's state. If the current
    /// action is completed, the result is evaluated and any new works is added to the pipeline to
    /// work on.
    pub fn tick_machine_state(&mut self, context: &RuntimeContext) -> Result<bool, Error> {
        // Get the current action and see if it is done.
        if self.current_action.is_none() {
            return Ok(true);
        }

        // Get the current state of the action.
        let action = self
            .current_action
            .as_mut()
            .expect("Action does not exist despite just checking its value");
        let is_done = action.is_done(context)?;

        // If the current action is still going, we can just bail here.
        if !is_done {
            return Ok(false);
        }

        // If the current action is done, we need to pop the next action and start it.
        // If the pipeline is done, we can return that we are done.
        let result = action.get_result(context);
        let should_cancel_pending_actions = match result {
            ActionResult::Success => false,
            ActionResult::FailedAllow => false,
            ActionResult::Failed => true,
            ActionResult::Canceled => true,
        };
        if should_cancel_pending_actions {
            self.pipeline.cancel()
        }

        // If there is new work, we will push these items onto the pipeline.
        if let Some(actions) = action.get_new_work(context) {
            for action in actions {
                self.pipeline.add_immediate_action(action);
            }
        }

        // We will dequeue the next action and start it (if there is any).
        self.current_action = self.pipeline.pop_next_action();
        let has_remaining_actions = self.current_action.is_some();
        if has_remaining_actions {
            let action = self.current_action.as_mut().unwrap();
            action.start(context)?;
        }
        Ok(has_remaining_actions)
    }
}
