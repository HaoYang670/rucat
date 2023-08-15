use std::thread;
use std::thread::Result;

pub mod configs;

/** TODO:
  1. formally verified the order is preserved if both `tasks` and returned are ordered
  3. limit the number of threads to be `cpus - 1` ?
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
