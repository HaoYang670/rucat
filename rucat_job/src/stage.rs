use serde_traitobject as st;

use crate::task::{SubDataTrait, SubExecute, SubResultTrait};

/// Driver will call `get_data` and `split`, and send `Task` to workers.
/// Workers simplify `SubTask`s to `SubResult`s and send them back to the driver.
/// Driver collects `SubResult`s and call `merge` to get the final result.
/// This is the counterpart of the Stage in Apache Spark.
pub struct Stage<Input, SubInput, SubOutput, Output> {
    /// Split `Data` into several pieces.
    /// The number of pieces should always be equals to the number of workers.
    split: Box<dyn Fn(Input, usize) -> Vec<SubInput>>,
    /// Size of a closure cannot be known at compile time. And we don't want to copied it which would take much memory
    execute: st::Rc<SubExecute>,
    merge: Box<dyn Fn(Vec<SubOutput>) -> Output>,
}

impl<Input, SubInput, SubOutput, Output> Stage<Input, SubInput, SubOutput, Output>
where
    SubInput: SubDataTrait + 'static, // the lifetime must be static because it is serializable (I tried to use 'a but failed)
    SubOutput: SubResultTrait,
{
    pub fn new(
        split: Box<dyn Fn(Input, usize) -> Vec<SubInput>>,
        execute: st::Rc<SubExecute>,
        merge: Box<dyn Fn(Vec<SubOutput>) -> Output>,
    ) -> Self {
        Self {
            split,
            execute,
            merge,
        }
    }

    // get the tasks that will be sent to workers
    //pub fn get_tasks(self, workers: usize) -> Vec<Task> {
    //    (self.split)(self.data, workers)
    //        .into_iter()
    //        .map(|sub_data| Task::new(st::Box::new(sub_data), self.execute.clone()))
    //        .collect::<Vec<Task>>()
    //}
}
