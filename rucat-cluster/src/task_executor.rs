/// We want Rucat can execute all kinds of tasks.
/// One choice is to define Task as an Enum to list
/// the types we support (sql, s-expression, ...). But it is not extendable
/// and it is not `forall tasks`.
/// Insteads, we define a trait here. any type of task which implements this trait
/// can be executed by Rucat.
/// 
/// [SubTask] and [SubResult] should be serializable.
/// This definition is now only a one-stage mode (not like map-reduce which has shuffle and multi-stages)
pub trait TaskExecutor<Task, SubTask, SubResult, Result> {
  /// Rucat driver splits the task into several sub tasks.
  fn split(task: Task) -> Vec<SubTask>;

  /// Rucat worker executes the sub task.
  fn execute(sub_task: SubTask) -> SubResult;

  /// Rucat driver collects sub results to get the final result.
  fn collect(sub_results: Vec<SubResult>) -> Result;
}