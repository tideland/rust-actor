// --------------------------------------------------------
// Actor library
// Copyright (C) 2024 Frank Mueller / Oldenburg / Europe / World
// --------------------------------------------------------

use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

// Define the result type for tasks and actor state.
pub type TaskResult = Result<(), String>;

#[derive(Debug, Clone)]
pub struct AsyncActor {
    sender: mpsc::Sender<Box<dyn FnOnce() -> TaskResult + Send>>,
    state: Arc<Mutex<TaskResult>>,
}

impl AsyncActor {
    // Creates a new AsyncActor with an initial Running state.
    pub fn new() -> Arc<Self> {
        let (sender, mut receiver) = mpsc::channel::<Box<dyn FnOnce() -> TaskResult + Send>>(32);
        let state = Arc::new(Mutex::new(Ok(())));

        let actor = Arc::new(Self {
            sender,
            state: state.clone(),
        });

        tokio::spawn(async move {
            while let Some(task) = receiver.recv().await {
                let task_result = task();
                let mut state_guard = state.lock().unwrap();

                // Update the state based on the task result.
                *state_guard = task_result.clone();

                // If a task results in an error, stop processing further tasks.
                if task_result.is_err() {
                    break;
                }
            }
        });

        actor
    }

    // Sends a task to the AsyncActor and returns the current state.
    pub async fn send<F>(&self, task: F) -> TaskResult
    where
        F: FnOnce() -> TaskResult + Send + 'static,
    {
        {
            // Check the current state before enqueuing a new task.
            let state_guard = self.state.lock().unwrap();
            if state_guard.is_err() {
                // If the actor is in an error state, return the error immediately.
                return state_guard.clone();
            }
        } // Release the lock before proceeding.

        let send_result = self.sender.send(Box::new(task)).await;
        match send_result {
            Ok(_) => {
                // Return the current state after enqueuing the task.
                self.state.lock().unwrap().clone()
            }
            Err(_) => self.state(),
        }
    }

    // Retrieves the current state of the AsyncActor.
    pub fn state(&self) -> TaskResult {
        self.state.lock().unwrap().clone()
    }
}

// --------------------------------------------------------
// EOF
// --------------------------------------------------------
