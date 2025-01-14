use ::std::borrow::Cow;

use ::serde::{Deserialize, Serialize};

/// States of Rucat engine
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
