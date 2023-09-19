use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    vec,
};

use crate::{
    communication::Task,
    error::{Result, RucatError},
};

/**
 * Workers are to execute sub tasks.
 * A worker can be assigned to different drivers to execute different tasks. 
 */
struct Worker {
    listener: TcpListener,
    // `None` if the worker is free, `Some(_)` if it is assigned to a worker
    stream: Option<TcpStream>,
}

impl Worker {
    /// Create a worker that can be bound to a driver.
    fn create_free_worker(addr: SocketAddr) -> Result<Self> {
        let listener = TcpListener::bind(addr).map_err(|err| err.to_string())?;
        Ok(Self{ listener, stream: None})
    }

    /// The worker is binded to a driver or not
    fn is_free(&self) -> bool {
        self.stream.is_none()
    }

    /// Bind the worker to a driver
    fn bind_driver(&mut self, driver_addr: SocketAddr) -> Result<()> {
        if self.is_free() {
            let (stream, addr) = self.listener.accept()?;
            if addr == driver_addr {
                self.stream = Some(stream);
                Ok(())
            } else {
                Err(RucatError::Other(format!(
                    "Accept message from unknown host {}",
                    addr
                )))
            }
        } else {
            Err(RucatError::Other("Worker is not free".to_owned()))
        }
    }

    /**
     * Worker reads the info from tcp stream into a small buffer.
     * The small buffer will append to a large buffer.
     * We parse [Task]s from the large buffer.
     */
    fn execute_tasks(&mut self) -> Result<()> {
        let stream =  self.stream.as_mut().ok_or(RucatError::Other("Worker is free".to_owned()))?;
        let mut large_buf: Vec<u8> = vec![];

        loop {
            let mut small_buf = [0; 1024];
            // Read data from the client
            let bytes_read = stream
                .read(&mut small_buf)
                .expect("Failed to read data from client");
            // Deserialize the received JSON data into a Task struct
            let received_data = &small_buf[..bytes_read];
            large_buf.append(&mut received_data.to_vec());
            match Task::deserialize_many(&large_buf)? {
                None => (),
                Some((tasks, remained)) => {
                    let tasks = tasks.to_vec();
                    tasks
                        .into_iter()
                        .for_each(|t| println!(" Received task from client: {}", t.get_task()));
                    large_buf = remained;
                    // Respond to the client
                    let response = "Task received";
                    stream.write_all(response.as_bytes())?
                }
            }
        }
    }
}
