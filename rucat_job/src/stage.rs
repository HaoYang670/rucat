use serde_traitobject as st;

use crate::task::{SubDataTrait, SubExecute, SubResultTrait, Task};

/// We want Rucat to execute all kinds of tasks. (turing complete)
/// One choice is to define Task as a sum type of all
/// the types we support (SQL, s-expression, python, ...). But it is not extendable and it is not the `forall tasks` we expect.
/// -- Although SQL and Python are turing complete expressions, it is not friendly for users to express all tasks in some
/// -- specific languages. Sometimes, it could be so complicate.
/// Insteads, we define a generic type here which the elements are functions of how to execute the task.
///
/// Driver will call `get_data` and `split`, and send `SubTask` to workers.
/// Workers simplify `SubTask`s to `SubResult`s and send them back to the driver.
/// Driver collects `SubResult`s and call `merge` to get the final result.
/// This is one-stage task (rename it to Stage?)
pub struct Stage<Data, SubData, SubResult, Result> {
    /// The `data` could be a constant value or a file path (e.x. you store the data on s3 bucket)
    data: Data,
    /// Split `Data` into several pieces.
    /// The number of pieces should always be equals to the number of workers.
    split: Box<dyn Fn(Data, usize) -> Vec<SubData>>,
    /// Size of a closure cannot be known at compile time. And we don't want to copied it which would take much memory
    execute: st::Rc<SubExecute>,
    merge: Box<dyn Fn(Vec<SubResult>) -> Result>,
}

impl<Data, SubData, SubResult, Result> Stage<Data, SubData, SubResult, Result>
where
    SubData: SubDataTrait + 'static, // the lifetime must be static because it is serializable (I tried to use 'a but failed)
    SubResult: SubResultTrait,
{
    pub fn new(
        data: Data,
        split: Box<dyn Fn(Data, usize) -> Vec<SubData>>,
        execute: st::Rc<SubExecute>,
        merge: Box<dyn Fn(Vec<SubResult>) -> Result>,
    ) -> Self {
        Self {
            data,
            split,
            execute,
            merge,
        }
    }

    /// get the sub tasks that will be sent to workers
    /// This is the only way to get a [`SubTask`]
    /// This function consumes the `Task`.
    pub fn get_sub_tasks(self, workers: usize) -> Vec<Task> {
        (self.split)(self.data, workers)
            .into_iter()
            .map(|sub_data| Task::new(st::Box::new(sub_data), self.execute.clone()))
            .collect::<Vec<Task>>()
    }
}
