// --------------------------------------------------------
// Actor library
// Copyright (C) 2024 Frank Mueller / Oldenburg / Europe / World
// --------------------------------------------------------

use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// ActorState represents the current state of the actor.
#[derive(Debug, Clone, PartialEq)]
pub enum ActorState {
    Running,
    Stopped,
    Error,
}

/// AsyncActor helps to run tasks asynchronously. Tasks are enqueued and processed
/// by the actor loop. The actor can be stopped at any time ensuring that all
/// tasks in the queue are processed before stopping.
///
/// Tasks are functions and closures taking no arguments and return a Result<(), String>.
/// The actor will stop processing tasks if an error is returned. All logical errors
/// have to be handled by the task itself or in the calling code, e.g. by using the
/// individual closure's error handling.
pub struct AsyncActor {
    sender: mpsc::Sender<Box<dyn FnOnce() -> Result<(), String> + Send>>,
    state: Arc<Mutex<ActorState>>,
    message: Arc<Mutex<Option<String>>>,
}

impl AsyncActor {
    /// Creates a new AsyncActor.
    pub fn new() -> Arc<Self> {
        let (sender, mut receiver) =
            mpsc::channel::<Box<dyn FnOnce() -> Result<(), String> + Send>>(32);
        let state = Arc::new(Mutex::new(ActorState::Running));
        let message = Arc::new(Mutex::new(None));

        let actor = Arc::new(Self {
            sender,
            state: state.clone(),
            message: message.clone(),
        });

        tokio::spawn(async move {
            while let Some(task) = receiver.recv().await {
                match task() {
                    Ok(()) => {}
                    Err(err_msg) => {
                        if err_msg == "ACTOR::STOP" {
                            *state.lock().unwrap() = ActorState::Stopped;
                            // Set the message to "Actor stopped" if it is not set yet.
                            if message.lock().unwrap().is_none() {
                                *message.lock().unwrap() = Some("Actor stopped".to_string());
                            }
                            break;
                        }
                        *state.lock().unwrap() = ActorState::Error;
                        *message.lock().unwrap() = Some(err_msg);
                        break;
                    }
                }
            }
        });

        actor
    }

    /// Sends a task to the AsyncActor.
    pub async fn send<F>(&self, task: F) -> Result<(), String>
    where
        F: FnOnce() -> Result<(), String> + Send + 'static,
    {
        {
            // Check the current state before enqueuing a new task.
            let state_guard = self.state.lock().unwrap();
            match *state_guard {
                ActorState::Running => {}
                ActorState::Stopped => return Err("Actor is stopped".to_string()),
                ActorState::Error => {
                    if let Some(msg) = &*self.message.lock().unwrap() {
                        return Err(msg.clone());
                    }
                }
            }
        } // Release the lock before proceeding.

        // Send the task to the actor loop.
        match self.sender.send(Box::new(task)).await {
            Ok(_) => {
                return Ok(());
            }
            Err(err_msg) => {
                return Err(format!("Actor send error: {}", err_msg.to_string()).to_string());
            }
        }
    }

    /// Retrieves the current state of the AsyncActor.
    pub fn state(&self) -> ActorState {
        self.state.lock().unwrap().clone()
    }

    /// Retrieves the current message of the AsyncActor.
    pub fn message(&self) -> Option<String> {
        self.message.lock().unwrap().clone()
    }

    /// Stops the actor. This method will return immediately while the actor will
    /// continue processing the remaining tasks in the queue before stopping.
    pub async fn stop(&self) -> Result<(), String> {
        let stopper = Box::new(|| Err("ACTOR::STOP".to_string()));
        match self.sender.send(stopper).await {
            Ok(_) => {
                return Ok(());
            }
            Err(err_msg) => {
                return Err(format!("Actor stopped: {}", err_msg.to_string()).to_string());
            }
        }
    }
}

// --------------------------------------------------------
// EOF
// --------------------------------------------------------
