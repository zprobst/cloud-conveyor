//! The Idea is to define a variety of entry point as well as some core logic that exists outside of any implementation of the 
//! service to a cloud provider or deployment mechanism. Instead, this is a very high level operational code  and the "over all"
//! logic of the service.

use crate::{Applcation, Trigger};

struct WebhookRequest { }
struct WebhookEvent {} 

pub trait WebhookInterpretor {

    fn interpret_event(req: &WebhookRequest) -> Vec<WebhookEvent>  {

    }

}

/// Takes an event and performs whatver the event defines on it.
pub fn handle_web_hook_event<T: WebhookInterpretor> (interpretor: T, request: &WebhookRequest) -> {

    // Take a look at the event and process it into a standard event enum.
    let events = interpretor.interpret_event(request);

    // Match on the enum event and evaluate the tiggers based on the application in question.
    for event in events{ 
        
    }

    // For each trigger that is matched, do the stuff required by that trigger as another job to enqueue.

}
