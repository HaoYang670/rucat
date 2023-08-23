use std::process::Command;

// Clients-server
// This is like the Spark master

fn main() {
    println!("Hello, world!");
    create_cluster((), &[()]);
}
// the cluster driver
type Driver = ();

// anything that can run rust code
type Machine = ();

/*
 * Create a driver and zero or more workers. (zero worker means local mode)
 * Return the Driver so that users can run program on it.
 *
 * I deside to use socket to achieve the communication between driver and executors.
 * So driver should know addresses of all executors and all executors should know driver's address.
 */
fn create_cluster(driver: Machine, workers: &[Machine]) -> Driver {
    println!("Create one driver with {} workers", workers.len());
    let driver = Command::new("./rucat-cluster/target/debug/rucat-cluster")
        .arg("driver")
        .output()
        .unwrap();
    let worker = Command::new("./rucat-cluster/target/debug/rucat-cluster")
        .arg("worker")
        .output()
        .unwrap();

    match (driver.status.success(), worker.status.success()) {
        (false, _) => println!("driver error: {}", String::from_utf8_lossy(&driver.stderr)),
        (_, false) => println!("worker error: {}", String::from_utf8_lossy(&worker.stderr)),
        (true, true) => println!(
            "driver opt: {}\nworker opt: {}",
            String::from_utf8_lossy(&driver.stdout),
            String::from_utf8_lossy(&worker.stdout)
        ),
    }
}

/*
 * Delete the cluster (driver and workers). (driver should know its workers)
 */
// fn delete_cluster(driver: Driver) -> bool {
//     todo!()
// }
