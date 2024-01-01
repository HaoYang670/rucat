use clap::Parser;
use rucat_cluster::configs::*;

// driver-workers

/**
 * We should define a driver to schedule and assign tasks
 * How to serialize a closure? `serde_closure`?
 * We need to define some functions (map, reduce, filter ...) to parallize and merge tasks (or to define a functor directly)
 * Remote servers: how to do it? tokio? -- P0
 *  toy: create a driver and a worker. (locally, 2 processes)
 *      driver sends the worker a number.
 *      worker prints it out and returns num + 1.
 * Interprocess communication: Socket? Message queue?
 * How to handle error?
 *
 * Driver and workers are independent processes (Is there a more stateless way? pure fn, or just treat a process as a function)
 * Driver and workers intersect by socket (driver sent tasks to workers and workers return back the result)
 * So, each process should bind to a TCP port?
 * And we need a strong typed communication protocal.
 */
fn main() {
    let cli = Cli::parse();
    println!("create 1 driver and {} workers", cli.workers)
}

fn execute_tasks() {
    println!("this is an executor")
}
