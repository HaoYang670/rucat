use std::{collections::LinkedList, time::Instant};

use crate::worker::execute;

// I need a macro to duplicate the Fn.
macro_rules! vec_duplicate {
  ($elem:expr; $type:ty; $n:expr;) => {{
      let mut temp: Vec<$type> = vec![];
      (0..$n).for_each(|_| temp.push($elem));
      temp
  }};
}

/**
 * A non trivial porblem is how the driver will be used. Where do tasks come from?
 * Driver process has an IP process binded which end users can write closures in their crate and send it to the driver.
 * We should provide a library for sending tasks (to driver).
 */
pub fn schedule_tasks() {
  println!("this is a driver");
  println!(
      "physical cpus = {}, logical cpus = {}",
      num_cpus::get_physical(),
      num_cpus::get()
  );
  let start = Instant::now();

  let tasks = vec_duplicate![
    Box::new(move || {
      (0..900).map(|i| (i as f32) / f32::EPSILON).sum::<f32>();
      start.elapsed()
    });
    Box<dyn Fn() -> _ + Send>;
    10;
  ];
  let result: LinkedList<_> = execute(tasks);
  println!("{:?}", result)
}
