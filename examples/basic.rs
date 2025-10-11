use xarxa_sdk_rs::prelude::*;
use xarxa_sdk_rs::prelude::exports;

// Example: Booking saga with compensations
struct Booking {
    flight_id: String,
    user_id: String,
    amount: f64,
}

fn reserve_flight(flight_id: &str) -> Result<(), String> {
    // Simulate flight reservation
    let msg = format!("run reserve flight with flight_id: {flight_id}");
    logging(msg.as_bytes());

    Ok(())
}

fn release_flight(_flight_id: &str) -> Result<(), String> {
    // Simulate flight release
    Ok(())
}

fn charge_card(_user_id: &str, _amount: f64) -> Result<(), String> {
    // Simulate card charge
    Ok(())
}

fn refund(_user_id: &str, _amount: f64) -> Result<(), String> {
    // Simulate refund
    Ok(())
}

fn send_email(_user_id: &str) -> Result<(), String> {
    // Simulate email sending
    Ok(())
}

fn send_sms(_user_id: &str) -> Result<(), String> {
    // Simulate SMS sending
    Ok(())
}

/// Example booking flow with saga pattern
fn booking_flow(b: Booking) -> PipelineState {
    // Clone the data we need for each closure
    let flight_id = b.flight_id.clone();
    let flight_id_comp = b.flight_id.clone();
    let user_id = b.user_id.clone();
    let user_id_comp = b.user_id.clone();
    let user_id_email = b.user_id.clone();
    let user_id_sms = b.user_id.clone();
    let amount = b.amount;

    Pipeline::new()
        .step("reserve", move |_ctx| reserve_flight(&flight_id))
        .compensate("reserve", move |_ctx| release_flight(&flight_id_comp))
        .step("charge", move |_ctx| charge_card(&user_id, amount))
        .compensate("charge", move |_ctx| refund(&user_id_comp, amount))
        .step("email_notify", move |_ctx| send_email(&user_id_email))
        .step("sms_notification", move |_ctx| send_sms(&user_id_sms))
        .build()
}

/// Example with context usage for data sharing between steps
fn payment_flow_with_context() -> PipelineState {
    Pipeline::new()
        .step("validate", |ctx| {
            // Validate and store in context
            ctx.set("payment_id", Value::Str("PAY-123".to_string()));
            ctx.set("amount", Value::Float64(99.99));
            Ok(())
        })
        .step("process", |ctx| {
            // Read from context
            let _payment_id = ctx.get_string("payment_id")
                .ok_or("Missing payment_id")?;
            let _amount = ctx.get_f64("amount")
                .ok_or("Missing amount")?;

            // Process payment
            ctx.set("status", Value::Str("processed".to_string()));
            Ok(())
        })
        .compensate("process", |ctx| {
            // Compensate using context data
            let _payment_id = ctx.get_string("payment_id").unwrap();
            // Refund logic here
            Ok(())
        })
        .step("notify", |ctx| {
            let _status = ctx.get_string("status").unwrap_or_default();
            // Send notification
            Ok(())
        })
        .build()
}

struct Engine;

impl Guest for Engine {
    fn start_workflow(workflow_name: String, input: Vec<u8>) -> Result<WorkflowRun, WorkflowError> {
        let msg = format!("Starting workflow: {}", workflow_name);
        logging(msg.as_bytes());

        // Store the original input for history
        let original_input = input.clone();

        // Create the appropriate workflow based on the name
        let mut pipeline = match workflow_name.as_str() {
            "booking" => {
                let booking = Booking {
                    flight_id: "FL-123".to_string(),
                    user_id: "user-456".to_string(),
                    amount: 299.99,
                };
                booking_flow(booking)
            },
            "payment" => payment_flow_with_context(),
            _ => {
                return Err(WorkflowError {
                    err: format!("Unknown workflow: {}", workflow_name),
                    code: Code::NotFound,
                });
            }
        };

        // Parse input data and add to context
        if !input.is_empty() {
            // You could parse JSON here, for example:
            // let input_data: serde_json::Value = serde_json::from_slice(&input)?;
            pipeline.context.set("raw_input", Value::Any(input));
        }

        // Generate a unique run ID
        let run_id = 12345u64; // In real implementation, this would be generated
        pipeline.context = pipeline.context.with_run_id(run_id);

        // Execute the first tick to start the workflow
        pipeline.tick();

        // Capture the current state as history
        let history = pipeline.capture_history(workflow_name.clone(), original_input);

        // Serialize the history to return to the host
        let history_bytes = serde_json::to_vec(&history)
            .map_err(|e| WorkflowError {
                err: format!("Failed to serialize history: {}", e),
                code: Code::EmptyInput,
            })?;

        let msg = format!(
            "Workflow started, executed {} steps",
            history.executed_steps.len()
        );
        logging(msg.as_bytes());

        // Package the history bytes as Kvpair to return to the host
        let history_kvpair = vec![Kvpair {
            key: "workflow_history".to_string(),
            value: Value::Any(history_bytes),
        }];

        // Return the workflow run with serialized history
        Ok(WorkflowRun {
            id: run_id,
            input: history_kvpair, // Return the history as Kvpair
            timestamp: 1640995200, // Unix timestamp - in real impl, use current time
        })
    }

    fn continue_workflow(id: RunId, state: Vec<u8>) -> Result<Vec<u8>, WorkflowError> {
        let msg = format!("Continuing workflow with run_id: {}", id);
        logging(msg.as_bytes());

        // Deserialize the history from the state
        let history: WorkflowHistory = serde_json::from_slice(&state)
            .map_err(|e| WorkflowError {
                err: format!("Failed to deserialize history: {}", e),
                code: Code::EmptyInput,
            })?;

        let msg = format!(
            "Loaded history for workflow '{}' with {} executed steps",
            history.workflow_name,
            history.executed_steps.len()
        );
        logging(msg.as_bytes());

        // Recreate the workflow based on the workflow name
        let mut pipeline = match history.workflow_name.as_str() {
            "booking" => {
                let booking = Booking {
                    flight_id: "FL-123".to_string(),
                    user_id: "user-456".to_string(),
                    amount: 299.99,
                };
                booking_flow(booking)
            },
            "payment" => payment_flow_with_context(),
            _ => {
                return Err(WorkflowError {
                    err: format!("Unknown workflow: {}", history.workflow_name),
                    code: Code::NotFound,
                });
            }
        };

        // Replay the workflow from history
        // This restores the context and step states without re-executing actions
        pipeline.replay_from_history(&history);

        // Execute the next tick to continue from where we left off
        pipeline.tick();

        // Capture the updated history
        let updated_history = pipeline.capture_history(
            history.workflow_name.clone(),
            history.original_input.clone(),
        );

        // Serialize the updated history
        let history_bytes = serde_json::to_vec(&updated_history)
            .map_err(|e| WorkflowError {
                err: format!("Failed to serialize updated history: {}", e),
                code: Code::EmptyInput,
            })?;

        let msg = format!(
            "Workflow continued, now at step {} with {} total executed steps",
            updated_history.current_step,
            updated_history.executed_steps.len()
        );
        logging(msg.as_bytes());

        // Return the updated history
        Ok(history_bytes)
    }
}

export!(Engine);
