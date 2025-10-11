// use xarxa_sdk_rs::pipeline::{
//     PipelineState, Step, StepResult, WorkflowContext, steps,
// };
// use xarxa_sdk_rs::{
//     bindings::orchestrator::export,
//     exports,
//     Value,
//     exports::xarxa::api::workflow_ctrl,
//     Code,
// };
// use xarxa_sdk_rs::bindings::orchestrator::exports::xarxa::api::workflow_ctrl;




use xarxa_sdk_rs::prelude::*;
use xarxa_sdk_rs::prelude::exports;


/// A realistic workflow that processes user data through multiple steps
fn data_processing_workflow() -> PipelineState {
    PipelineState::new(vec![
        // Step 0
        steps::log_to_host("start_log", "Starting workflow"),

        // Step 1: Set up initial data
        steps::set_value("setup", "api_url", Value::Str("https://jsonplaceholder.typicode.com/users/1".to_string())),

        // Step 2: Log what we're about to do
        steps::log("start_log", "Starting workflow: fetching user data from {api_url}"),

        // Step 3: Make HTTP request to fetch user data
        steps::http_get("fetch_user", "api_url", "user_response"),

        // Step 4: Parse and transform the response
        steps::transform("parse_user", |ctx| {
            let response = ctx.get_bytes("user_response")
                .ok_or("No user response found")?;

            // Simulate parsing JSON (in real use you'd use serde_json)
            let response_str = String::from_utf8(response.clone())
                .map_err(|_| "Invalid UTF-8 in response")?;

            // Extract some basic info (mock parsing)
            let user_id = if response_str.contains("\"id\":") {
                1234 // Mock extracted ID
            } else {
                0
            };

            ctx.set("user_id", Value::Int64(user_id));
            ctx.set("raw_response", Value::Str(response_str));

            Ok(())
        }),

        // Step 5: Conditional logic based on user data
        steps::condition("check_user", |ctx| {
            let user_id = ctx.get_i64("user_id").unwrap_or(0);

            if user_id > 0 {
                Ok(StepResult::Continue) // Proceed to success path
            } else {
                Ok(StepResult::JumpTo("error_handler".to_string())) // Jump to error handling
            }
        }),

        // Step 6: Calculate some metrics
        steps::calculate("calculate_metrics", "score", |ctx| {
            let user_id = ctx.get_i64("user_id").unwrap_or(0) as f64;
            let score = user_id * 1.5 + 42.0; // Some arbitrary calculation
            Ok(score)
        }),

        // Step 7: Log success
        steps::log("success_log", "User {user_id} processed successfully with score {score}"),

        // Step 8: Skip error handler
        Step::new("skip_error", |_ctx| Ok(StepResult::Skip(1))),

        // Step 9: Error handler (will be skipped if everything goes well)
        Step::new("error_handler", |ctx| {
            ctx.set("error_message", Value::Str("Failed to process user data".to_string()));
            Ok(StepResult::Continue)
        }).retry(3), // Retry up to 3 times

        // Step 10: Final logging
        steps::log("final_log", "Workflow completed. Final status: {error_message}"),
    ])
}

/// Another example: E-commerce order processing workflow
fn order_processing_workflow() -> PipelineState {
    PipelineState::new(vec![
        // Initialize order data
        steps::transform("init_order", |ctx| {
            ctx.set("order_id", Value::Str("ORD-12345".to_string()));
            ctx.set("customer_id", Value::Int64(67890));
            ctx.set("total_amount", Value::Float64(99.99));
            Ok(())
        }),

        // Validate order
        steps::condition("validate_order", |ctx| {
            let amount = ctx.get_f64("total_amount").unwrap_or(0.0);
            if amount > 0.0 {
                ctx.set("status", Value::Str("validated".to_string()));
                Ok(StepResult::Continue)
            } else {
                Ok(StepResult::JumpTo("cancel_order".to_string()))
            }
        }),

        // Check inventory (simulate HTTP call to inventory service)
        steps::transform("check_inventory", |ctx| {
            // Mock inventory check - in real workflow this would be an HTTP request
            let available = true; // Simulate inventory check result
            ctx.set("inventory_available", Value::Boolean(available));

            if !available {
                return Err("Item out of stock".to_string());
            }
            Ok(())
        }).retry(2), // Retry inventory checks

        // Process payment
        steps::transform("process_payment", |ctx| {
            let amount = ctx.get_f64("total_amount").unwrap_or(0.0);
            // Simulate payment processing
            ctx.set("payment_id", Value::Str(format!("PAY-{}", (amount * 100.0) as u64)));
            ctx.set("payment_status", Value::Str("success".to_string()));
            Ok(())
        }),

        // Ship order
        steps::transform("ship_order", |ctx| {
            let order_id = ctx.get_string("order_id").unwrap_or_default();
            ctx.set("tracking_number", Value::Str(format!("TRK-{}-001", order_id)));
            ctx.set("status", Value::Str("shipped".to_string()));
            Ok(())
        }),

        // Send confirmation
        steps::log("send_confirmation", "Order {order_id} shipped! Tracking: {tracking_number}"),

        // Skip cancellation step
        Step::new("skip_cancel", |_ctx| Ok(StepResult::Skip(1))),

        // Cancellation handler
        Step::new("cancel_order", |ctx| {
            ctx.set("status", Value::Str("cancelled".to_string()));
            Ok(StepResult::Continue)
        }),

        // Final status
        steps::log("final_status", "Order processing complete. Status: {status}"),
    ])
}


struct Engine;

impl Guest for Engine {
    fn start_workflow(workflow_name: String, input: Vec<u8>) -> Result<WorkflowRun, WorkflowError> {
        // Create the appropriate workflow based on the name
        let mut pipeline = match workflow_name.as_str() {
            "data_processing" => data_processing_workflow(),
            "order_processing" => order_processing_workflow(),
            _ => {
                return Err(WorkflowError {
                    err: format!("Unknown workflow: {}", workflow_name),
                    code: Code::NotFound,
                });
            }
        };

        // Parse input data (assuming it's JSON or some serialized format)
        // For this example, we'll just use the raw bytes
        if !input.is_empty() {
            // You could parse JSON here, for example:
            // let input_data: serde_json::Value = serde_json::from_slice(&input)?;
            pipeline.context.set("raw_input", Value::Any(input));
        }

        // Generate a unique run ID
        let run_id = 12345u64; // In real implementation, this would be generated
        pipeline.context = pipeline.context.with_run_id(run_id);

        // Execute the first tick to get started
        pipeline.tick();

        // Note: In a real implementation, you'd serialize the pipeline state
        // For this example, we'll skip serialization due to function pointer complexity

        // Create the workflow run result
        Ok(WorkflowRun {
            id: run_id,
            input: vec![], // Could include processed input here
            timestamp: 1640995200, // Unix timestamp - in real impl, use current time
        })
    }

    fn continue_workflow(id: RunId, _state: Vec<u8>) -> Result<Vec<u8>, WorkflowError> {
        // In a real implementation, you'd deserialize the pipeline state from `state`
        // For this example, we'll create a new pipeline to demonstrate the concept

        let mut pipeline = data_processing_workflow()
            .with_context(WorkflowContext::new().with_run_id(id));

        // Execute one tick of the workflow
        pipeline.tick();

        // Return empty state for now (in real impl, serialize the updated pipeline)
        Ok(vec![])
    }
}

export!(Engine);
