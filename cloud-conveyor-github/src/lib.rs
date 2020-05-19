//! This crate defines the webhook interpretor for github webhook requests.
//!  Due to the interface for the WebhookInterpretor trait, this implementation
//! requires that all of the error cases are handled in here. If invalid request bodies are
//! sent for instance, we will simply return an empty vector of events.
//!
//! This crate also uses the standard logging pattern that the core crate uses.
