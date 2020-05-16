//! This crate defines the webhook interpretor for github webhook requests.
//!  Due to the interface for the WebhookInterpretor trait, this implementation
//! requires that all of the error cases are handled in here. If invalid request bodies are
//! sent for instance, we will simply return an empty vector of events.
//!
//! This crate also uses the standard logging pattern that the core crate uses.
use cloud_conveyor_core::webhook::{WebhookEvent, WebhookInterpretor, WebhookRequest};

pub struct GithubWebhook {}

impl WebhookInterpretor for GithubWebhook {
    fn interpret_webhook_payload(&self, _: &WebhookRequest) -> Vec<WebhookEvent> {
        // Validate the request is signed properly

        // Serialize it into the appropriate type.

        // If that type means that there is a vcs event that we are expected to yield,
        // we can add it to the result vec with the right information.
        todo!()
    }
}
