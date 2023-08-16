fn main() {
    println!("Hello, world!");
}
// the cluster driver
// type Driver = ();

// anything that can run rust code
// type Machine = ();

/*
 * Create a driver and zero or more executors. (zero executor means local mode)
 * Return the Driver so that users can run program on it.
 */
//fn create_cluster(driver: Machine, executors: &[Machine]) -> Driver {
//    todo!()
//}

/*
 * Delete the cluster (driver and executors). (driver should know its executors)
 */
// fn delete_cluster(driver: Driver) -> bool {
//     todo!()
// }
