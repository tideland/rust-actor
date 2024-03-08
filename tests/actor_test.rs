// --------------------------------------------------------
// Actor library - Tests
// Copyright (C) 2024 Frank Mueller / Oldenburg / Europe / World
// --------------------------------------------------------

use actor::{ActorState, AsyncActor};
use std::sync::{Arc, Mutex};

#[tokio::test]
// Test the async actor with a simple positive task.
async fn test_actor() {
    let actor = AsyncActor::new();

    // Send a task to the actor.
    let result = actor.send(|| Ok(())).await;

    assert_eq!(result, Ok(()));
}

#[tokio::test]
// Test the async actor with a simple positive task and stopping it.
async fn test_actor_stop() {
    let actor = AsyncActor::new();

    let _ = actor.stop().await;

    // Sadly we have to wait a bit to ensure that the actor is stopped
    // as it works asynchronously.
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    assert_eq!(
        actor.state(),
        ActorState::Stopped,
        "Actor should be in stopped state"
    );

    // Expect sending a task to the actor to fail.
    let result = actor.send(|| Ok(())).await;
    assert_eq!(result, Err("Actor is stopped".to_string()));
}

#[tokio::test]
// Test an error task. All tasks after error talk should not be processed.
async fn test_actor_error() {
    let actor = AsyncActor::new();
    // Send initial error task to the actor.
    let _ = actor.send(|| Err("Ouch!".to_string())).await;
    let counter = Arc::new(Mutex::new(1));

    loop {
        let counter_clone = counter.clone();
        let result = actor
            .send(move || {
                let mut counter = counter_clone.lock().unwrap();
                *counter += 1;
                Ok(())
            })
            .await;
        // Check the current value of counter outside the closure
        let current_counter = *counter.lock().unwrap();
        if result.is_err() || current_counter > 50 {
            break;
        }
    }
    let test_counter = *counter.lock().unwrap();

    assert_eq!(test_counter, 1, "Counter should be 0");
    assert_eq!(
        actor.state(),
        ActorState::Error,
        "Actor should be in error state"
    )
}

#[tokio::test]
// Test an actor as field of a struct.
async fn test_shared_async_actor() {
    let test_actor = AsyncCounter::new().await;

    test_actor.incr().await;
    test_actor.incr().await;
    test_actor.incr().await;
    test_actor.decr().await;

    // Wait a bit to ensure that the actor has processed all tasks.
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let value = test_actor.read_value().await;

    assert_eq!(value, 2, "Counter should be 2");
}

// --------------------------------------------------------
// TEST HELPER
// --------------------------------------------------------

// AsyncCounter helps testing using the AsyncActor inside a struct as
// a field.
struct AsyncCounter {
    actor: Arc<AsyncActor>,
    value: Arc<Mutex<i32>>,
}

impl AsyncCounter {
    async fn new() -> Self {
        let actor = AsyncActor::new();
        let value = Arc::new(Mutex::new(0));
        AsyncCounter { actor, value }
    }

    async fn incr(&self) {
        let value = self.value.clone();
        let actor = self.actor.clone();

        let _ = actor
            .send(move || {
                let mut value = value.lock().unwrap();
                *value += 1;
                Ok(())
            })
            .await;
    }

    async fn decr(&self) {
        let value = self.value.clone();
        let actor = self.actor.clone();

        let _ = actor
            .send(move || {
                let mut value = value.lock().unwrap();
                *value -= 1;
                Ok(())
            })
            .await;
    }

    async fn read_value(&self) -> i32 {
        *self.value.lock().unwrap()
    }
}

/*
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct CounterState {
    value: i32,
    ops: u64,
}

struct Counter {
    state: Arc<Mutex<CounterState>>,
    actor: Arc<Actor>,
}

impl Counter {
    async fn new(actor: Arc<Actor>) -> Self {
        let state = Arc::new(Mutex::new(CounterState::default()));
        Counter { state, actor }
    }

    async fn incr(&self) {
        let state = self.state.clone();
        let actor = self.actor.clone();

        actor
            .send_async(move || {
                let mut state = state.lock().unwrap();
                state.value += 1;
                state.ops += 1;
                Ok(())
            })
            .await;
    }

    async fn decr(&self) {
        let state = self.state.clone();
        let actor = self.actor.clone();

        actor
            .send_async(move || {
                let mut state = state.lock().unwrap();
                state.value -= 1;
                state.ops += 1;
                Ok(())
            })
            .await;
    }

    async fn read_value(&self) -> i32 {
        let actor = self.actor.clone();
        let state_for_closure = self.state.clone();
        let state_for_reading = self.state.clone(); // Clone again for use after the closure.

        // Now use `state_for_closure` inside the closure.
        actor
            .send_sync(Box::new(move || {
                let mut state = state_for_closure.lock().unwrap();
                state.ops += 1;
                Ok(())
            }))
            .await
            .expect("Failed to send read task");

        // Use `state_for_reading` here.
        let state = state_for_reading.lock().unwrap();
        state.value
    }
}
*/
// --------------------------------------------------------
// EOF
// --------------------------------------------------------
