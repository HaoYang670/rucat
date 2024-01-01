use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use serde_traitobject as st;

// Debug trait is for the tarpc
pub trait SubDataTrait: st::Serialize + st::Deserialize + Debug {}
pub trait SubResultTrait: st::Serialize + st::Deserialize + Debug {}
pub trait SubExecuteTrait: st::Fn<SubData, Output = SubResult> + Debug {}

type SubData = (st::Box<dyn SubDataTrait>,);
pub type SubResult = st::Box<dyn SubResultTrait>;
pub type SubExecute = st::Box<dyn SubExecuteTrait>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    sub_data: st::Box<dyn SubDataTrait>,
    execute: st::Rc<SubExecute>,
}

impl Task {
    pub fn new(sub_data: st::Box<dyn SubDataTrait>, execute: st::Rc<SubExecute>) -> Self {
        Task { sub_data, execute }
    }

    /// Simplify the `SubTask` into `SubResult`
    /// This consumes the `SubTask`
    pub fn simplify(self) -> SubResult {
        (self.execute)(self.sub_data)
    }
}
