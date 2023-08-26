use serde::{Deserialize, Serialize};
use crate::error::Result;

/// Delimiter between tasks
const DELIMITER: u8 = b'\n';

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Task {
  task: u8,
}

impl Task {
  pub fn get_task(&self) -> u8 {
    self.task
  }

  /**
   * serialize [Task] and append a [DELIMITER]
   */
  pub fn serialize(&self) -> Result<Vec<u8>> {
    let mut buf = vec![];
    buf.append(&mut serde_json::to_vec(self)?);
    buf.append(&mut vec![DELIMITER]);
    Ok(buf)
  }

  /**
   * Parse a [Task] from the head of a buffer and returns the remained buffer.
   */
  pub fn deserialize(buf: Vec<u8>) -> (Option<Result<Self>>, Vec<u8>) {
    // find the delimiter
    match buf.iter().position(|&byte| byte == DELIMITER) {
      None => (None, buf),
      Some(i) => {
        let (parsed, remainder) = buf.split_at(i);
        let task = serde_json::from_slice(parsed);
        (Some(task.map_err(|e| e.into())), remainder[1..].to_vec())
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn deserialize_from_empty() {
    let buf = vec![];
    assert_eq!(Task::deserialize(buf), (None, vec![]));
  }

  #[test]
  fn deserialize_two() -> Result<()> {
    let task1 = Task{task: 1 };
    let task2 = Task{task: 2 };
    let mut buf = vec![];
    buf.append(&mut task1.serialize()?);
    buf.append(&mut task2.serialize()?);

    let (result1, buf) = Task::deserialize(buf);
    assert_eq!(result1.unwrap()?, task1);
    let (result2, buf) = Task::deserialize(buf);
    assert_eq!(result2.unwrap()?, task2);
    assert!(buf.is_empty());
    Ok(())
  }
}
