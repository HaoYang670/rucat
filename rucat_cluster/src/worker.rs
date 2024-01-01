use std::future::{self, Ready};

use tarpc::context::Context;

use rucat_job::task::{SubResult, SubTask};

/// Worker doesn't know the type of the task sent to it because it has been compiled.
/// We use trait object instead, as serde: https://github.com/alecmocatta/serde_traitobject
/// Driver sends a trait object [`SubInput`] to worker. Worker executes it and returns a trait object [`SubOutput`]
/// back to driver.
struct Worker {}

impl WorkerService for Worker {
    #[doc = "The response future returned by [`Work::execute`]."]
    type ExecuteFut = Ready<SubResult>;

    fn execute(self, _context: Context, sub_task: SubTask) -> Self::ExecuteFut {
        future::ready(sub_task.simplify())
    }
}

#[tarpc::service]
trait WorkerService {
    async fn execute(sub_task: SubTask) -> SubResult;
    // TODO: metrics for assess pressure
}
