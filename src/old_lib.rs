wit_bindgen::generate!({
    path: "wit/engine",
    world: "controller",
});

use crate::old_lib::xarxa::engine::host_controller;

use crate::old_lib::exports::xarxa::engine::worker_handler::Guest;
use crate::old_lib::xarxa::engine::engine_types::{History, Kvpair, RunId, Signal, Value, WorkflowError, WorkflowRun};

trait FromValue: Sized {
    fn from_value(v: &Value) -> Option<Self>;
}

impl FromValue for i64 {
    fn from_value(v: &Value) -> Option<Self> {
        if let Value::Int64(i) = v {
            Some(*i)
        } else {
            None
        }
    }
}

impl FromValue for u64 {
    fn from_value(v: &Value) -> Option<Self> {
        if let Value::Uint64(i) = v {
            Some(*i)
        } else {
            None
        }
    }
}

impl FromValue for f64 {
    fn from_value(v: &Value) -> Option<Self> {
        if let Value::Float64(i) = v {
            Some(*i)
        } else {
            None
        }
    }
}

impl FromValue for String {
    fn from_value(v: &Value) -> Option<Self> {
        if let Value::Str(i) = v {
            Some(i.clone())
        } else {
            None
        }
    }
}

impl FromValue for bool {
    fn from_value(v: &Value) -> Option<Self> {
        if let Value::Boolean(b) = v {
            Some(*b)
        } else {
            None
        }
    }
}

struct Engine;

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let p = match self {
            Value::None => "none",
            Value::Str(v) => &v,
            Value::Int64(v) => &v.to_string(),
            Value::Uint64(v) => &v.to_string(),
            Value::Float64(v) => &v.to_string(),
            Value::Boolean(v) => &v.to_string(),
            Value::Any(v) => std::str::from_utf8(v).unwrap_or("invalid"),
        };

        write!(f, "{p:?}")
    }
}

impl Guest for Engine {
    #[doc = " Start a new workflow from scratch. Passes the name of the workflow and input data."]
    #[doc = " The history here is always empty."]
    #[allow(async_fn_in_trait)]
    fn start_workflow(workflow_name: String, input: Vec::<Kvpair>,) -> Result<WorkflowRun, WorkflowError> {
        Ok(WorkflowRun {
            id: 123,
            input,
            timestamp: 5554234234,
        })
    }
    
    #[doc = " Continue to execute an “asleep” workflow. This is the most frequent command."]
    #[doc = " Transfers the entire event history so that SDK can execute replay."]
    #[allow(async_fn_in_trait)]
    fn continue_workflow(id: RunId, _history: History,) -> Result<String, WorkflowError> {
        todo!()
    }
    
    #[doc = " Execute a specific function-activity. Passes the name of the activity and its input data."]
    #[doc = " This command is “dumb”, it does not deal with the replay logic."]
    #[allow(async_fn_in_trait)]
    fn execute_activity(activity_name: String, input: Vec::<Kvpair>,) -> Result<String, String> {
        todo!()
    }
    
    #[doc = " “Push” the signal into the workflow. Passes the name of the signal and its payload."]
    #[allow(async_fn_in_trait)]
    fn signal_workflow(id: RunId, signal: Signal,) -> Result<_rt::String,WorkflowError> {
        todo!()
    }
    
    #[doc = " Notify the running workflow of a cancel request."]
    #[doc = " This is essentially just sending a predefined internal __cancel signal."]
    #[allow(async_fn_in_trait)]
    fn cancel_workflow(id:RunId,) -> Result<_rt::String,WorkflowError> {
        todo!()
    }
}


// fn convert<T: FromValue>(v: &Value) -> Option<T> {
//     T::from_value(v)
// }

fn my_function() {
    let run_id: RunId = 123;
    let task_result: &Kvpair = &Kvpair { key: "step_123".to_string(), value: Value::Str("str".to_string()) };

    let _result = host_controller::send_result(run_id, task_result);
}

export!(Engine);