// --------------------------------------------------------
// Actor library - Tests
// Copyright (C) 2024 Frank Mueller / Oldenburg / Europe / World
// --------------------------------------------------------

use actor::AsyncActor;

#[tokio::test]
// Test the async actor with a simple task. Will always return Ok(())
// as the task is processed asynchonously. So sending an error task
// first might return Err() as well as Ok() depending on how early
// it will be processed.
async fn test_actor() {
    let actor = AsyncActor::new();

    // Send a task to the actor.
    let result = actor.send(|| Ok(())).await;

    assert_eq!(result, Ok(()));
}

// Test the async actor with a simple task that fails. It will stop processing
// further tasks. Initial positiv messages have to be sent before the error
// due to queueing. Still the error task is the first to be processed and so
// it's error later.
#[tokio::test]
async fn test_actor_error_loop() {
    let actor = AsyncActor::new();
    let ouch_str = "Ouch!".to_string();
    let ouch_err = Err(ouch_str.clone());
    // Send initial error task to the actor.
    let _ = actor.send(|| ouch_err).await;

    let mut ret_str = "init".to_string();
    let mut counter = 50;

    println!(
        "'{}' is expected to be changed in the test to '{}'",
        ret_str, ouch_str,
    );

    loop {
        // Should once return the first error as it is the result of the first task.
        let result = actor.send(|| Ok(())).await;
        counter -= 1;

        if counter == 0 || result.is_err() {
            match result {
                Ok(_) => {
                    ret_str = "ok".to_string();
                }
                Err(e) => {
                    ret_str = e;
                }
            }
            break;
        }
    }

    assert!(counter > 0, "Counter should be greater than 0");
    assert_eq!(ret_str, ouch_str, "Expected error string not found");
}

// --------------------------------------------------------
// TEST HELPER
// --------------------------------------------------------
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
