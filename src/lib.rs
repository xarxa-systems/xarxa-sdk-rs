pub mod pipeline;
// pub mod ctrl;
pub mod bindings;

pub mod prelude {
    pub use crate::bindings::orchestrator::export;
    pub use crate::bindings::orchestrator::exports;

    pub use crate::bindings::orchestrator::exports::xarxa::api::workflow_ctrl::Guest;
    pub use crate::bindings::orchestrator::xarxa::api::engine_types::{
        WorkflowRun, WorkflowError, RunId, Value, Kvpair, Code,
    };

    pub use crate::pipeline::{PipelineState, Step, StepResult, WorkflowContext, steps};
}

// #[macro_export]
// macro_rules! register_workflow {
//     ($ty:ident) => {
//         $crate::bindings::orchestrator::exports::xarxa::api::workflow_ctrl::__export_xarxa_api_workflow_ctrl_0_1_1_cabi!(
//             $ty with_types_in $crate::bindings::orchestrator::exports::xarxa::api::workflow_ctrl
//         );
//     };
// }


// // Create a public macro that generates the necessary export functions
// #[macro_export]
// macro_rules! exc{
//     ($ty:ident) => {
//         const _: () = {
//             #[unsafe(export_name = "xarxa:api/workflow-ctrl@0.1.1#start-workflow")]
//             unsafe extern "C" fn export_start_workflow(arg0: *mut u8,arg1: usize,arg2: *mut u8,arg3: usize,) -> *mut u8 {
//                 unsafe {
//                     $crate::bindings::orchestrator::exports::xarxa::api::workflow_ctrl::_export_start_workflow_cabi::<$ty>(arg0, arg1, arg2, arg3)
//                 }
//             }

//             #[unsafe(export_name = "cabi_post_xarxa:api/workflow-ctrl@0.1.1#start-workflow")]
//             unsafe extern "C" fn _post_return_start_workflow(arg0: *mut u8,) {
//                 unsafe {
//                     $crate::bindings::orchestrator::exports::xarxa::api::workflow_ctrl::__post_return_start_workflow::<$ty>(arg0)
//                 }
//             }

//             #[unsafe(export_name = "xarxa:api/workflow-ctrl@0.1.1#continue-workflow")]
//             unsafe extern "C" fn export_continue_workflow(arg0: i64,arg1: *mut u8,arg2: usize,) -> *mut u8 {
//                 unsafe {
//                     $crate::bindings::orchestrator::exports::xarxa::api::workflow_ctrl::_export_continue_workflow_cabi::<$ty>(arg0, arg1, arg2)
//                 }
//             }

//             #[unsafe(export_name = "cabi_post_xarxa:api/workflow-ctrl@0.1.1#continue-workflow")]
//             unsafe extern "C" fn _post_return_continue_workflow(arg0: *mut u8,) {
//                 unsafe {
//                     $crate::bindings::orchestrator::exports::xarxa::api::workflow_ctrl::__post_return_continue_workflow::<$ty>(arg0)
//                 }
//             }
//         };
//     };
// }

// use proc_macro::TokenStream;
// use quote::quote;
// use syn::{parse_macro_input, Ident};

// pub use bindings::orchestrator::exports::xarxa::api::workflow_ctrl::Guest;
 
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

// #[proc_macro]
// pub fn register(input: TokenStream) -> TokenStream {
//     let fn_name = parse_macro_input!(input as Ident);

//     let expanded = quote! {
//         struct XarxaWorkflow;

//         impl durable_workflow::workflow::Guest for XarxaWorkflow {
//             fn workflow_init(_input: ::std::vec::Vec<u8>) -> ::std::vec::Vec<u8> {
//                 let plan = #fn_name();
//                 let state = xarxa_sdk::PipelineState::from(plan);
//                 bincode::serialize(&state).unwrap()
//             }

//             fn workflow_tick(state_bytes: ::std::vec::Vec<u8>) -> ::std::vec::Vec<u8> {
//                 let mut st: xarxa_sdk::PipelineState = bincode::deserialize(&state_bytes).unwrap();
//                 st.tick();
//                 bincode::serialize(&st).unwrap()
//             }

//             fn workflow_finished(state_bytes: ::std::vec::Vec<u8>) -> bool {
//                 let st: xarxa_sdk::PipelineState = bincode::deserialize(&state_bytes).unwrap();
//                 st.is_finished()
//             }
//         }

//         durable_workflow::workflow::export!(XarxaWorkflow);
//     };

//     TokenStream::from(expanded)
// }


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
