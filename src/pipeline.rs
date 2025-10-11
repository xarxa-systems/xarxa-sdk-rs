use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::bindings::orchestrator::xarxa::api::{
    engine_types::{Kvpair, RunId, Value},
    logger::logging,
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

/// Serializable representation of a Value
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SerializableValue {
    None,
    Str(String),
    Int64(i64),
    Uint64(u64),
    Float64(f64),
    Boolean(bool),
    Any(Vec<u8>),
}

impl SerializableValue {
    pub fn from_value(value: &Value) -> Self {
        match value {
            Value::None => SerializableValue::None,
            Value::Str(s) => SerializableValue::Str(s.clone()),
            Value::Int64(i) => SerializableValue::Int64(*i),
            Value::Uint64(u) => SerializableValue::Uint64(*u),
            Value::Float64(f) => SerializableValue::Float64(*f),
            Value::Boolean(b) => SerializableValue::Boolean(*b),
            Value::Any(bytes) => SerializableValue::Any(bytes.clone()),
        }
    }

    pub fn to_value(&self) -> Value {
        match self {
            SerializableValue::None => Value::None,
            SerializableValue::Str(s) => Value::Str(s.clone()),
            SerializableValue::Int64(i) => Value::Int64(*i),
            SerializableValue::Uint64(u) => Value::Uint64(*u),
            SerializableValue::Float64(f) => Value::Float64(*f),
            SerializableValue::Boolean(b) => Value::Boolean(*b),
            SerializableValue::Any(bytes) => Value::Any(bytes.clone()),
        }
    }
}

/// Serializable version of context for history
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SerializableContext {
    pub data: HashMap<String, SerializableValue>,
}

impl SerializableContext {
    pub fn from_context(ctx: &HashMap<String, Value>) -> Self {
        let data = ctx.iter()
            .map(|(k, v)| (k.clone(), SerializableValue::from_value(v)))
            .collect();
        SerializableContext { data }
    }

    pub fn to_context(&self) -> HashMap<String, Value> {
        self.data.iter()
            .map(|(k, v)| (k.clone(), v.to_value()))
            .collect()
    }
}

/// Captures the result of a single step execution in the history
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StepExecutionRecord {
    pub step_name: String,
    pub status: StepStatus,
    pub error: Option<String>,
    /// Context state after this step completed (serialized)
    pub context_snapshot: SerializableContext,
}

/// Complete execution history for replay
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkflowHistory {
    pub workflow_name: String,
    pub original_input: Vec<u8>,
    pub run_id: RunId,
    /// Steps that have been executed in order
    pub executed_steps: Vec<StepExecutionRecord>,
    /// Current step index in the workflow
    pub current_step: usize,
    /// Final context state (serialized)
    pub context: SerializableContext,
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

pub struct PipelineState {
    pub steps: Vec<Step>,
    pub current: usize,
    pub context: WorkflowContext,
    /// Track execution history for replay
    pub execution_history: Vec<StepExecutionRecord>,
}

impl PipelineState {
    pub fn new(steps: Vec<Step>) -> Self {
        PipelineState {
            steps,
            current: 0,
            context: WorkflowContext::new(),
            execution_history: Vec::new(),
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

        let current_step_idx = self.current;

        // Check the step status first
        let step_status = self.steps[current_step_idx].status.clone();

        match step_status {
            StepStatus::Pending => {
                self.steps[current_step_idx].status = StepStatus::Running;

                let message = "Pending -> Running".to_string();
                logging(&message.into_bytes());
            }
            StepStatus::Running => {
                // Execute the action - need to extract it to avoid borrow conflicts
                let action_result = {
                    let step = &self.steps[current_step_idx];
                    (step.action)(&mut self.context)
                };

                match action_result {
                    Ok(result) => {
                        self.steps[current_step_idx].status = StepStatus::Completed;

                        // Record the successful execution
                        self.record_step_execution(current_step_idx);

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
                                self.steps[current_step_idx].status = StepStatus::Pending;
                            }
                        }
                    }
                    Err(e) => {
                        let retries_left = self.steps[current_step_idx].retries_left;

                        if retries_left > 0 {
                            self.steps[current_step_idx].retries_left -= 1;
                            self.steps[current_step_idx].status = StepStatus::Pending;
                        } else {
                            self.steps[current_step_idx].status = StepStatus::Failed(e.clone());

                            // Record the failed execution
                            self.record_step_execution(current_step_idx);

                            // Check if there's a compensation function
                            let has_compensation = self.steps[current_step_idx].compensation.is_some();

                            if has_compensation {
                                self.steps[current_step_idx].status = StepStatus::Compensating;

                                // Execute compensation
                                let comp_result = {
                                    let step = &self.steps[current_step_idx];
                                    if let Some(comp) = &step.compensation {
                                        comp(&mut self.context)
                                    } else {
                                        Ok(())
                                    }
                                };

                                match comp_result {
                                    Ok(_) => {
                                        self.steps[current_step_idx].status = StepStatus::Compensated;
                                        self.record_step_execution(current_step_idx);
                                        self.current += 1;
                                    }
                                    Err(_err) => {
                                        // Compensation failed, workflow is stuck
                                        self.record_step_execution(current_step_idx);
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

    /// Replay workflow from history without executing actions
    /// This reconstructs the context and step states from previous executions
    pub fn replay_from_history(&mut self, history: &WorkflowHistory) {
        // Restore the context from history
        self.context.data = history.context.to_context();
        self.context.run_id = Some(history.run_id);

        // Set the current step position
        self.current = history.current_step;

        // Replay each executed step to restore state
        for (idx, record) in history.executed_steps.iter().enumerate() {
            if idx < self.steps.len() {
                self.steps[idx].status = record.status.clone();

                // Log the replay
                let msg = format!(
                    "Replaying step '{}' with status: {:?}",
                    record.step_name, record.status
                );
                logging(msg.as_bytes());
            }
        }

        // Copy the execution history
        self.execution_history = history.executed_steps.clone();

        let msg = format!(
            "Replayed {} steps, continuing from step {}",
            history.executed_steps.len(),
            self.current
        );
        logging(msg.as_bytes());
    }

    /// Capture current execution state as history
    pub fn capture_history(&self, workflow_name: String, original_input: Vec<u8>) -> WorkflowHistory {
        WorkflowHistory {
            workflow_name,
            original_input,
            run_id: self.context.run_id.unwrap_or(0),
            executed_steps: self.execution_history.clone(),
            current_step: self.current,
            context: SerializableContext::from_context(&self.context.data),
        }
    }

    /// Record a step execution in history
    fn record_step_execution(&mut self, step_index: usize) {
        if step_index < self.steps.len() {
            let step = &self.steps[step_index];
            let record = StepExecutionRecord {
                step_name: step.name.clone(),
                status: step.status.clone(),
                error: match &step.status {
                    StepStatus::Failed(e) => Some(e.clone()),
                    _ => None,
                },
                context_snapshot: SerializableContext::from_context(&self.context.data),
            };

            // Update or append the record
            if step_index < self.execution_history.len() {
                self.execution_history[step_index] = record;
            } else {
                self.execution_history.push(record);
            }
        }
    }
}

/// Builder for creating pipelines with a fluent API and saga pattern support
pub struct Pipeline {
    steps: Vec<StepBuilder>,
}

struct StepBuilder {
    name: String,
    action: Box<dyn Fn(&mut WorkflowContext) -> Result<(), String> + Send + Sync>,
    compensation: Option<Box<dyn Fn(&mut WorkflowContext) -> Result<(), String> + Send + Sync>>,
    max_retries: u32,
}

impl Pipeline {
    /// Create a new empty pipeline
    pub fn new() -> Self {
        Pipeline { steps: Vec::new() }
    }

    /// Add a step with an action that receives context
    /// Use `|_ctx|` if you don't need the context
    pub fn step<F>(mut self, name: &str, action: F) -> Self
    where
        F: Fn(&mut WorkflowContext) -> Result<(), String> + Send + Sync + 'static,
    {
        let step = StepBuilder {
            name: name.to_string(),
            action: Box::new(action),
            compensation: None,
            max_retries: 0,
        };
        self.steps.push(step);
        self
    }

    /// Add a compensation action for a specific step by name
    /// Use `|_ctx|` if you don't need the context
    pub fn compensate<F>(mut self, step_name: &str, compensation: F) -> Self
    where
        F: Fn(&mut WorkflowContext) -> Result<(), String> + Send + Sync + 'static,
    {
        if let Some(step) = self.steps.iter_mut().find(|s| s.name == step_name) {
            step.compensation = Some(Box::new(compensation));
        }
        self
    }

    /// Set the retry count for the last added step
    pub fn retry(mut self, count: u32) -> Self {
        if let Some(last_step) = self.steps.last_mut() {
            last_step.max_retries = count;
        }
        self
    }

    /// Build the pipeline into a PipelineState
    pub fn build(self) -> PipelineState {
        let steps: Vec<Step> = self.steps.into_iter().map(|sb| {
            let action = sb.action;
            
            Step {
                name: sb.name,
                status: StepStatus::Pending,
                retries_left: sb.max_retries,
                max_retries: sb.max_retries,
                compensation: sb.compensation,
                action: Box::new(move |ctx| {
                    action(ctx)?;
                    Ok(StepResult::Continue)
                }),
            }
        }).collect();

        PipelineState::new(steps)
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Alias for the result of a pipeline builder function
pub type PipelinePlan = Pipeline;
