use std::{
    io,
    net::{TcpListener, TcpStream},
};

/// Driver is a node which splits a task into subtasks,
/// assigns subtasks to workers, gets sub results back from workers and merges sub results into the final result.
/// When assigning a task to a driver (the cluster manager's duty), workers should also be assigned together
/// so that driver can know the number of workers and how many sub tasks should be splitted.
/// Driver should own Workers so that the linear type system can make one Worker can only has one driver.
pub struct Driver {
    listener: TcpListener,
    workers: Vec<TcpStream>, // tcp stream or address?
}

impl Driver {
    ///
    fn assign_workers() {}
    ///
    fn unassign_workers() {}
}

/**
 * A non trivial porblem is how the driver will be used. Where do tasks come from?
 * Driver process has an IP process binded which end users can write closures in their crate and send it to the driver.
 * We should provide a library for sending tasks (to driver).
 */
pub fn schedule_tasks() {
    println!("this is a driver");
    println!("receiving tasks: ");

    loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                println!("receive: {}", input);
            }
            Err(error) => {
                println!("Error reading input: {}", error);
            }
        }
    }
}
