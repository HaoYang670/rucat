//use tarpc::context;

/// Worker doesn't know the type of the task sent to it because it has been compiled.
/// We use trait object instead, as serde: https://github.com/alecmocatta/serde_traitobject
/// We need to add trait bound for SubData and SubResult. Both of them should be serde.

//#[tarpc::service]
trait Worker {
    //async fn execute(t: SubTask) -> SubResult;
}
