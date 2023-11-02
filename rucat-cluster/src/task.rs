use serde::{Deserialize, Serialize};
use serde_traitobject as st;

pub trait SubDataTrait: st::Serialize + st::Deserialize {}
pub trait SubResultTrait: st::Serialize + st::Deserialize {}

type SubInput = (st::Box<dyn SubDataTrait>,);
type SubOutput = st::Box<dyn SubResultTrait>;

/// We want Rucat to execute all kinds of tasks.
/// One choice is to define Task as a sum type of all
/// the types we support (sql, s-expression, python, ...). But it is not extendable and it is not the `forall tasks` we expect.
/// Insteads, we define a generic type here which the elements are functions of how to execute the task.
///
/// Driver will call `get_data` and `split`, and send `SubTask` to workers.
/// Workers simplify `SubTask`s to `SubResult`s and send them back to the driver.
/// Driver collects `SubResult`s and call `merge` to get the final result.
pub struct Task<Data, SubData, SubResult, Result> {
    /// The `data` could be a constant value or a file path (e.x. you store the data on s3 bucket)
    data: Data,
    /// Split `Data` into several pieces.
    /// The number of pieces should always be equals to the number of workers.
    split: Box<dyn Fn(Data, usize) -> Vec<SubData>>,
    /// Size of a closure cannot be known at compile time. And we don't want to copied it which would take much memory
    execute: st::Rc<st::Box<dyn st::Fn<SubInput, Output = SubOutput>>>,
    merge: Box<dyn Fn(Vec<SubResult>) -> Result>,
}

impl<Data, SubData, SubResult, Result> Task<Data, SubData, SubResult, Result>
where
    SubData: SubDataTrait + 'static,
    SubResult: SubResultTrait,
{
    pub fn new(
        data: Data,
        split: Box<dyn Fn(Data, usize) -> Vec<SubData>>,
        execute: st::Rc<st::Box<dyn st::Fn<SubInput, Output = SubOutput>>>,
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
    pub fn get_sub_tasks(self, workers: usize) -> Vec<SubTask> {
        (self.split)(self.data, workers)
            .into_iter()
            .map(|sub_data| SubTask {
                sub_data: st::Box::new(sub_data),
                execute: self.execute.clone(),
            })
            .collect::<Vec<SubTask>>()
    }
}

#[derive(Serialize, Deserialize)]
pub struct SubTask {
    sub_data: st::Box<dyn SubDataTrait>,
    execute: st::Rc<st::Box<dyn st::Fn<SubInput, Output = SubOutput>>>,
}

impl SubTask {
    /// Simplify the `SubTask` into `SubResult`
    /// This consumes the `SubTask`
    pub fn simplify(self) -> SubOutput {
        (self.execute)(self.sub_data)
    }
}
