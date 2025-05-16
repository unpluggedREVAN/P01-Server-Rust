use std::sync::atomic::AtomicUsize;

pub mod task_queue;
pub mod endpoints;
pub mod responses;
pub mod error_responses;
pub mod handle_connection;
static CONNECTION_COUNT: AtomicUsize = AtomicUsize::new(0);