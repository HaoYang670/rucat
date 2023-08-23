use std::thread;
use std::thread::Result;

/** TODO:
  1. formally verified the order is preserved if both `tasks` and returned are ordered
*/
pub fn execute<TASKS, R, CONTAINER>(tasks: TASKS) -> CONTAINER
where
    TASKS: IntoIterator<Item = Box<dyn Fn() -> R + Send>>,
    R: Send + 'static,
    CONTAINER: FromIterator<Result<R>>,
{
    let handles = tasks
        .into_iter()
        .map(|t| thread::spawn(t))
        .collect::<Vec<_>>();

    handles.into_iter().map(|h| h.join()).collect()
}
