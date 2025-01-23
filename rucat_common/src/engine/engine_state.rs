use ::std::borrow::Cow;

use ::serde::{Deserialize, Serialize};

/// States of Rucat engine
/// The states can be described from 2 different perspectives:
/// 1. The state flow perspective:
/// - `WaitToStart` -> `TriggerStart` -> `StartInProgress` -> `Running`
/// - `WaitToTerminate` -> `TriggerTermination` -> `TerminateInProgress` -> `Terminated`
/// - `ErrorWaitToClean` -> `ErrorTriggerClean` -> `ErrorCleanInProgress` -> `ErrorClean`
/// 2. The type of states perspective:
/// - waiting states: `WaitToStart`, `WaitToTerminate`, `ErrorWaitToClean`
/// - trigger states: `TriggerStart`, `TriggerTermination`, `ErrorTriggerClean`
/// - in progress states: `StartInProgress`, `Running`, `TerminateInProgress`, `ErrorCleanInProgress`
/// - stable states: `Terminated`, `ErrorClean`
///   `Running` is a special state that it is a `in progress` state because there are engine resources
///   associated with it, and engine resources are not stable.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum EngineState {
    WaitToStart,
    TriggerStart,
    StartInProgress,
    Running,
    WaitToTerminate,
    TriggerTermination,
    TerminateInProgress,
    Terminated,
    ErrorWaitToClean(Cow<'static, str>),
    ErrorTriggerClean(Cow<'static, str>),
    ErrorCleanInProgress(Cow<'static, str>),
    ErrorClean(Cow<'static, str>),
}
