/// Counterpart of TaskContext
pub(crate) enum TaskContext {
    /// Task context with extra contextual info and tooling for tasks in a barrier stage.
    BarrierTaskContext,
    /// ???
    TaskContextImpl,
}

impl TaskContext {}
