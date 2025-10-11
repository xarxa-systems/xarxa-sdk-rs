pub use crate::bindings::orchestrator::exports::xarxa::api::workflow_ctrl::Guest;
use crate::bindings::orchestrator::xarxa::api::engine_types::{Kvpair, RunId, Value, WorkflowError, WorkflowRun};

use crate::pipeline::Step;



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

// pub struct Engine{
//     pub(crate) steps: Vec<Step>,
// }

// impl Engine {
//     pub fn new(steps: Vec<Step>) -> Self {
//         Engine{steps}
//     }

//     // pub fn start(&self, workflow_name: String, input: Vec<Kvpair>) -> Result<WorkflowRun, WorkflowError> {
//     //     self. (workflow_name, input)
//     // }
// }


// impl Guest for Engine {
//     #[allow(async_fn_in_trait)]
//     fn start_workflow(workflow_name: String, input: Vec::<u8>,) -> Result<WorkflowRun,WorkflowError> {
//         todo!()
//     }
    
//     #[allow(async_fn_in_trait)]
//     fn continue_workflow(id:RunId, state:Vec::<u8>,) -> Result<Vec::<u8>,WorkflowError> {
//         todo!()
//     }
    
//     // type WorkflowEngine = Engine;

//     // fn create_workflow_engine() -> WorkflowEngine {
//     //     WorkflowEngine::new(Engine::new(vec![]))
//     // }
// }


// impl GuestWorkflowEngine for Engine {
//     #[allow(async_fn_in_trait)]
//     fn new() -> Self {
//         Engine{ steps: vec![] }
//     }

//     #[doc = " Start a new workflow from scratch. Passes the name of the workflow and input data."]
//     #[doc = " The history here is always empty."]
//     #[allow(async_fn_in_trait)]
//     fn start_workflow(&self, _workflow_name: String, input: Vec::<Kvpair>) -> Result<WorkflowRun, WorkflowError> {

//         let s = self.steps.first().expect("start workflow err");

//         let (step_name, step_fn) = s;

//         let res = step_fn();

//         Ok(WorkflowRun {
//             id: 123,
//             input,
//             timestamp: 5554234234,
//         })
//     }
    
//     #[doc = " Continue to execute an “asleep” workflow. This is the most frequent command."]
//     #[doc = " Transfers the entire event history so that SDK can execute replay."]
//     #[allow(async_fn_in_trait)]
//     fn continue_workflow(&self, id: RunId, _history: History) -> Result<String, WorkflowError> {
//         todo!()
//     }
    
//     #[doc = " Execute a specific function-activity. Passes the name of the activity and its input data."]
//     #[doc = " This command is “dumb”, it does not deal with the replay logic."]
//     #[allow(async_fn_in_trait)]
//     fn execute_activity(&self, activity_name: String, input: Vec::<Kvpair>,) -> Result<String, String> {
//         todo!()
//     }
    
//     #[doc = " “Push” the signal into the workflow. Passes the name of the signal and its payload."]
//     #[allow(async_fn_in_trait)]
//     fn signal_workflow(&self, id: RunId, signal: Signal,) -> Result<String,WorkflowError> {
//         todo!()
//     }
    
//     #[doc = " Notify the running workflow of a cancel request."]
//     #[doc = " This is essentially just sending a predefined internal __cancel signal."]
//     #[allow(async_fn_in_trait)]
//     fn cancel_workflow(&self, id: RunId) -> Result<String,WorkflowError> {
//         todo!()
//     }
    
//     #[doc(hidden)]
//     unsafe fn _resource_new(val: *mut u8) -> u32 where Self:Sized {
//         #[cfg(target_arch = "wasm32")]
//         #[link(wasm_import_module = "[export]xarxa:api/workflow-ctrl@0.1.1")]
//         unsafe extern "C" {
//             #[link_name = "[resource-new]workflow-engine"]
//             fn new(_: *mut u8,) -> i32;
        
//             }#[cfg(not(target_arch = "wasm32"))]
//         unsafe extern "C" fn new(_: *mut u8,) -> i32 {
//             unreachable!()
//         }
//         unsafe {
//             new(val)as u32
//         }
//     }
    
//     #[doc(hidden)]
//     fn _resource_rep(handle:u32) ->  *mut u8 where Self:Sized {
//         #[cfg(target_arch = "wasm32")]
//         #[link(wasm_import_module = "[export]xarxa:api/workflow-ctrl@0.1.1")]
//         unsafe extern "C" {
//             #[link_name = "[resource-rep]workflow-engine"]
//             fn rep(_:i32,) ->  *mut u8;
        
//             }#[cfg(not(target_arch = "wasm32"))]
//         unsafe extern "C" fn rep(_:i32,) ->  *mut u8 {
//             unreachable!()
//         }
//         unsafe {
//             rep(handle as i32)
//         }
//     }
// }


fn my_function() {
    let run_id: RunId = 123;
    let message = "step_123: str".to_string();

    logger::logging(run_id, &message.into_bytes());
}

// export!(Engine);