use std::{
    io,
    net::{TcpListener, TcpStream},
};

///
pub struct Driver {
    listener: TcpListener,
    workers: Vec<TcpStream>, // tcp stream or address?
}
/*
impl DriverService for Driver {

    /// Driver should also be a server that receive tasks from cluster manager
    fn execute(task: Task) {

    }

    #[doc = "The response future returned by [`DriverService::execute`]."]
type ExecuteFut = Ready<>;
}

#[tarpc::service]
trait DriverService {
    async fn execute(task: Task) -> SubResult;
    // TODO: metrics for assess pressure
}
*/
