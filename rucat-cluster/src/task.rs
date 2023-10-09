use std::rc::Rc;

/// We want Rucat can execute all kinds of tasks.
/// One choice is to define Task as an Enum to list
/// the types we support (sql, s-expression, ...). But it is not extendable
/// and it is not `forall tasks`.
/// Insteads, we define a generic struct here which the elements are functions of how to execute the task.
/// 
/// Driver will call `get_data` and `split`, and send `(SubData, execute)` to workers.
/// Worker executes `execute` on `SubData` and sends `SubResult` back to Driver.
/// Driver collects `SubResult`s and call `merge` to get the final result.
pub struct Task<Data, SubData, SubResult, Result> {
  /// The `data` could be a constant value or a file path (e.x. you store the data on s3 bucket)
  data: Data,
  /// Split `Data` into several pieces.
  /// The number of pieces should always be equals to the number of workers.
  split: Box<dyn Fn(Data, usize) -> Vec<SubData>>,
  /// Size of a closure cannot be known at compile time. And we don't want to copied it which would take much memory
  execute: Rc<Box<dyn Fn(SubData) -> SubResult>>,
  merge: Box<dyn Fn(Vec<SubResult>) -> Result>,
}

impl<Data, SubData, SubResult, Result> Task<Data, SubData, SubResult, Result> {
  pub fn new(
    data: Data,
    split: Box<dyn Fn(Data, usize) -> Vec<SubData>>,
    execute: Rc<Box<dyn Fn(SubData) -> SubResult>>,
    merge: Box<dyn Fn(Vec<SubResult>) -> Result>,
  ) -> Self {
    Self {data, split, execute, merge}
  }

  /// get the sub tasks that will be sent to workers
  /// This is the only way to get a [`SubTask`]
  pub fn get_sub_tasks(&self, workers: usize) -> Vec<SubTask<SubData, SubResult>> {
    (self.split)(self.data, workers)
      .iter()
      .map(|sub_data| SubTask{sub_data: *sub_data, execute: self.execute.clone()})
      .collect::<Vec<SubTask<SubData, SubResult>>>()
  }
}

/// This is the type that Driver sends to Worker.
/// TODO: This type should be serialized / deserialized
pub struct SubTask<SubData, SubResult> {
  sub_data: SubData,
  /// Size of a closure cannot be known at compile time. And we don't want to copied it which would take much memory
  execute: Rc<Box<dyn Fn(SubData) -> SubResult>>,
}

impl<SubData, SubResult> SubTask<SubData, SubResult> {
  /// Call the `execute` function on `sub_data` to get sub result
  pub fn simplify(&self) -> SubResult {
    (self.execute)(self.sub_data)
  }
}