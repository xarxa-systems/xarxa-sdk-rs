use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::{
    bindings::orchestrator::xarxa::api::custom_http::http_request,
    bindings::orchestrator::xarxa::api::http_types::{Header, Method},
    bindings::orchestrator::xarxa::api::logger::logging,
    bindings::orchestrator::xarxa::api::engine_types::{Kvpair, RunId, Value},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
    Compensating,
    Compensated,
}

/// Workflow context that flows between steps
#[derive(Debug, Clone)]
pub struct WorkflowContext {
    pub data: HashMap<String, Value>,
    pub run_id: Option<RunId>,
}

impl WorkflowContext {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            run_id: None,
        }
    }

    pub fn with_run_id(mut self, run_id: RunId) -> Self {
        self.run_id = Some(run_id);
        self
    }

    pub fn set(&mut self, key: &str, value: Value) {
        self.data.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        match self.get(key)? {
            Value::Str(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn get_i64(&self, key: &str) -> Option<i64> {
        match self.get(key)? {
            Value::Int64(i) => Some(*i),
            _ => None,
        }
    }

    pub fn get_f64(&self, key: &str) -> Option<f64> {
        match self.get(key)? {
            Value::Float64(f) => Some(*f),
            _ => None,
        }
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.get(key)? {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn get_bytes(&self, key: &str) -> Option<&Vec<u8>> {
        match self.get(key)? {
            Value::Any(bytes) => Some(bytes),
            _ => None,
        }
    }
}

/// Result of a step execution
#[derive(Debug)]
pub enum StepResult {
    /// Continue to next step
    Continue,
    /// Skip the next N steps
    Skip(usize),
    /// Jump to a specific step by name
    JumpTo(String),
    /// Wait for external event (async operation)
    Wait(String), // reason for waiting
}

pub struct Step {
    pub name: String,
    pub status: StepStatus,
    pub retries_left: u32,
    pub max_retries: u32,
    pub compensation: Option<Box<dyn Fn(&mut WorkflowContext) -> Result<(), String> + Send + Sync>>,
    pub action: Box<dyn Fn(&mut WorkflowContext) -> Result<StepResult, String> + Send + Sync>,
}


impl Step {
    /// Create a new step with a context-aware action
    pub fn new<F>(name: &str, action: F) -> Self
    where
        F: Fn(&mut WorkflowContext) -> Result<StepResult, String> + Send + Sync + 'static,
    {
        Step {
            name: name.to_string(),
            status: StepStatus::Pending,
            retries_left: 0,
            max_retries: 0,
            compensation: None,
            action: Box::new(action),
        }
    }

    /// Create a simple step that just continues to the next step
    pub fn simple<F>(name: &str, action: F) -> Self
    where
        F: Fn(&mut WorkflowContext) -> Result<(), String> + Send + Sync + 'static,
    {
        Step::new(name, move |ctx| {
            action(ctx)?;
            
            Ok(StepResult::Continue)
        })
    }

    /// Create a step that doesn't need context (legacy compatibility)
    pub fn basic<F>(name: &str, action: F) -> Self
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        Step::simple(name, move |_ctx| action())
    }

    pub fn retry(mut self, n: u32) -> Self {
        self.max_retries = n;
        self.retries_left = n;
        self
    }

    pub fn compensate<F>(mut self, f: F) -> Self
    where
        F: Fn(&mut WorkflowContext) -> Result<(), String> + Send + Sync + 'static,
    {
        self.compensation = Some(Box::new(f));
        self
    }
}

pub struct PipelineState {
    pub steps: Vec<Step>,
    pub current: usize,
    pub context: WorkflowContext,
}

impl PipelineState {
    pub fn new(steps: Vec<Step>) -> Self {
        PipelineState {
            steps,
            current: 0,
            context: WorkflowContext::new(),
        }
    }

    pub fn with_context(mut self, context: WorkflowContext) -> Self {
        self.context = context;
        self
    }

    pub fn with_input(mut self, input: Vec<Kvpair>) -> Self {
        for kv in input {
            self.context.set(&kv.key, kv.value);
        }
        self
    }

    pub fn tick(&mut self) {
        if self.current >= self.steps.len() {
            // Send final result if we have a run_id
            if let Some(run_id) = self.context.run_id {
                let message = format!("workflow with run_id: {run_id} - completed");
                logging(&message.into_bytes());
            }
            return;
        }

        let step = &mut self.steps[self.current];

        match step.status {
            StepStatus::Pending => {
                step.status = StepStatus::Running;

                let message = "Pending -> Running".to_string();
                logging( &message.into_bytes());
            }
            StepStatus::Running => {
                match (step.action)(&mut self.context) {
                    Ok(result) => {
                        step.status = StepStatus::Completed;

                        // Handle the step result
                        match result {
                            StepResult::Continue => {
                                self.current += 1;
                            }
                            StepResult::Skip(n) => {
                                self.current += n + 1;
                            }
                            StepResult::JumpTo(step_name) => {
                                if let Some(pos) = self.steps.iter().position(|s| s.name == step_name) {
                                    self.current = pos;
                                } else {
                                    // Step not found, move to next
                                    self.current += 1;
                                }
                            }
                            StepResult::Wait(_reason) => {
                                // Stay on current step, will be resumed by external event
                                step.status = StepStatus::Pending;
                            }
                        }
                    }
                    Err(e) => {
                        if step.retries_left > 0 {
                            step.retries_left -= 1;
                            step.status = StepStatus::Pending;
                        } else {
                            step.status = StepStatus::Failed(e.clone());
                            if let Some(comp) = &step.compensation {
                                step.status = StepStatus::Compensating;
                                match comp(&mut self.context) {
                                    Ok(_) => {
                                        step.status = StepStatus::Compensated;
                                        self.current += 1;
                                    }
                                    Err(_err) => {
                                        // Compensation failed, workflow is stuck
                                    }
                                }
                            } else {
                                // No compensation, workflow fails
                                return;
                            }
                        }
                    }
                }
            }
            StepStatus::Completed | StepStatus::Compensated => {
                self.current += 1;
            }
            StepStatus::Failed(_) | StepStatus::Compensating => {}
        }
    }

    pub fn is_finished(&self) -> bool {
        self.current >= self.steps.len()
    }
}

/// Helper functions for creating common step types
pub mod steps {
    use super::*;

    /// Make an HTTP request and store the response
    pub fn http_get(name: &str, url_key: &str, response_key: &str) -> Step {
        let url_key = url_key.to_string();
        let response_key = response_key.to_string();
        Step::new(name, move |ctx| {
            let url = ctx.get_string(&url_key)
                .ok_or_else(|| format!("Missing URL in context key: {}", url_key))?;

            match http_request(&[], Method::Get, &url, &[]) {
                Ok(response) => {
                    ctx.set(&response_key, Value::Any(response));
                    Ok(StepResult::Continue)
                }
                Err(e) => Err(format!("HTTP request failed: {}", e))
            }
        })
    }

    /// Make an HTTP POST request with data from context
    pub fn http_post(name: &str, url_key: &str, data_key: &str, response_key: &str) -> Step {
        let url_key = url_key.to_string();
        let data_key = data_key.to_string();
        let response_key = response_key.to_string();
        Step::new(name, move |ctx| {
            let url = ctx.get_string(&url_key)
                .ok_or_else(|| format!("Missing URL in context key: {}", url_key))?;
            let data = ctx.get_bytes(&data_key)
                .ok_or_else(|| format!("Missing data in context key: {}", data_key))?;

            match http_request(&[], Method::Post, &url, data) {
                Ok(response) => {
                    ctx.set(&response_key, Value::Any(response));
                    Ok(StepResult::Continue)
                }
                Err(e) => Err(format!("HTTP POST failed: {}", e))
            }
        })
    }

    /// Transform data in the context
    pub fn transform<F>(name: &str, f: F) -> Step
    where
        F: Fn(&mut WorkflowContext) -> Result<(), String> + Send + Sync + 'static,
    {
        Step::simple(name, move |ctx| f(ctx))
    }

    /// Conditional step that can skip or jump based on context data
    pub fn condition<F>(name: &str, f: F) -> Step
    where
        F: Fn(&mut WorkflowContext) -> Result<StepResult, String> + Send + Sync + 'static,
    {
        Step::new(name, move |ctx| f(ctx))
    }

    /// Log a message with data from context
    pub fn log(name: &str, message_template: &str) -> Step {
        let template = message_template.to_string();

        Step::simple(name, move |ctx| {
            // Simple template replacement - replace {key} with context values
            let mut message = template.clone();

            for (key, value) in &ctx.data {
                let placeholder = format!("{{{}}}", key);
                let value_str = match value {
                    Value::Str(s) => s.clone(),
                    Value::Int64(i) => i.to_string(),
                    Value::Float64(f) => f.to_string(),
                    Value::Boolean(b) => b.to_string(),
                    Value::Uint64(u) => u.to_string(),
                    Value::None => "none".to_string(),
                    Value::Any(_) => "[binary data]".to_string(),
                };
                message = message.replace(&placeholder, &value_str);
            }

            // Send log to host if we have a run_id
            if let Some(run_id) = ctx.run_id {
                let log_msg = format!("{message}, run_id: {run_id}");

                logging(&log_msg.into_bytes());
            }

            Ok(())
        })
    }

    pub fn log_to_host(name: &str, message: &str) -> Step {
        let msg = message.to_string();

        Step::simple(name, move |_ctx| {
            logging(&msg.clone().into_bytes());

            Ok(())
        })
    }

    /// Set a value in the context
    pub fn set_value(name: &str, key: &str, value: Value) -> Step {
        let key = key.to_string();
        Step::simple(name, move |ctx| {
            ctx.set(&key, value.clone());
            Ok(())
        })
    }

    /// Calculate and store a numeric result
    pub fn calculate<F>(name: &str, result_key: &str, f: F) -> Step
    where
        F: Fn(&mut WorkflowContext) -> Result<f64, String> + Send + Sync + 'static,
    {
        let result_key = result_key.to_string();
        Step::simple(name, move |ctx| {
            let result = f(ctx)?;
            ctx.set(&result_key, Value::Float64(result));
            Ok(())
        })
    }
}

// pub type Step = (String, Box<dyn Fn() -> Result<(), String> + 'static>);

// pub struct Pipeline {
//     pub(crate) steps: Vec<Step>,
// }

// impl Pipeline {
//     pub fn new() -> Self {
//         Pipeline { steps: vec![] }
//     }

//     pub fn step<F>(mut self, name: &str, f: F) -> Self
//     where
//         F: Fn() -> Result<(), String> + 'static,
//     {
//         self.steps.push((name.to_string(), Box::new(f)));
//         self
//     }

// }

// pub trait Pipeline<F> {
//     fn step(&mut self, name: &str, f: F) -> Self;
// }

// enum StepType {
//     Action(String),
// }

// impl Pipeline for Engine {
//     // pub fn new() -> Engine {
//     //     Engine {
//     //         steps: Vec::new(),
//     //         id: todo!(),
//     //     }
//     // }

//     fn step<F>(&mut self, name: &str, f: F) -> Self
//     where
//         F: Fn() -> Result<(), String> + 'static,
//     {
//         self.steps.push((name.to_string(), Box::new(f)));
//         self
//     }

//     // fn run(self) -> Result<(), String> {
//     //     for (name, step) in self.steps {
//     //         println!("Executing step: {name}");
            
//     //         step()?;
//     //     }

//     //     Ok(())
//     // }
// }
