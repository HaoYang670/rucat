use std::{collections::LinkedList, time::Instant};

use clap::Parser;
use rucat_cluster::{configs::*, execute};
use num_cpus;

// I need a macro to duplicate the Fn.
macro_rules! vec_duplicate {
    ($elem:expr; $type:ty; $n:expr;) => {{
        let mut temp: Vec<$type> = vec![];
        (0..$n).for_each(|_| temp.push($elem));
        temp
    }};
}

/**
 * Rename the project: Rucat (rust + category)
 * We should define a driver to schedule and assign tasks
 * We need to define some functions (map, reduce, filter ...) to parallize and merge tasks (or to define a functor directly)
 * Remote servers: how to do it? tokio?
 * How to handle error?
 *
 * I need a category manager to setup drivers and executors
 */
fn main() {
    let cli = Cli::parse();
    match cli.mode {
        Role::Driver => schedule_tasks(),
        Role::Worker => execute_tasks(),
    }
}

fn schedule_tasks() {
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

fn execute_tasks() {
    println!("this is an executor")
}
