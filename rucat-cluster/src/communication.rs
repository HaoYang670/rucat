use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Delimiter between tasks
const DELIMITER: u8 = b'\n';

/// This Task is just for dummy testing.
/// Please look at [TaskExecutor] to see what a real task is.
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
     * Parse as many [Task]s as possible until it can't or meet an error
     *  Err(_): buf has invalid format.
     *  Ok(None): buf is incomplete.
     *  Ok(Some(_, _)): successfully parse one or more [Task]s.
     */
    pub fn deserialize_many(buf: &[u8]) -> Result<Option<(NonEmptyVec<Task>, Vec<u8>)>> {
        match Self::deserialize(buf)? {
            None => Ok(None),
            Some((hd, remainder)) => match Self::deserialize_many(&remainder)? {
                None => Ok(Some((NonEmptyVec::new(hd, vec![]), remainder))),
                Some((tl, remainder)) => Ok(Some((NonEmptyVec::new(hd, tl.to_vec()), remainder))),
            },
        }
    }

    /**
     * Parse a [Task] from the head of a buffer and returns the remained buffer.
     * Return:
     *  Err(_) : buf has invalid format.
     *  Ok(None): buf is incomplete
     *  Ok(Some(_, _)): successfully parse a [Task]
     */
    pub fn deserialize(buf: &[u8]) -> Result<Option<(Task, Vec<u8>)>> {
        // find the delimiter
        match buf.iter().position(|&byte| byte == DELIMITER) {
            None => Ok(None),
            Some(i) => {
                let (parsed, remainder) = buf.split_at(i);
                let task = serde_json::from_slice(parsed)?;
                Ok(Some((task, remainder[1..].to_vec())))
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct NonEmptyVec<T> {
    hd: T,
    tl: Vec<T>,
}

impl<T> NonEmptyVec<T> {
    pub fn new(hd: T, tl: Vec<T>) -> Self {
        Self { hd, tl }
    }

    pub fn to_vec(self) -> Vec<T> {
        let mut tasks = self.tl;
        tasks.insert(0, self.hd);
        tasks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_from_empty() {
        let buf = vec![];
        assert_eq!(Task::deserialize(&buf), Ok(None));
        assert_eq!(Task::deserialize_many(&buf), Ok(None));
    }

    #[test]
    fn deserialize_two() -> Result<()> {
        let task1 = Task { task: 1 };
        let task2 = Task { task: 2 };
        let mut buf = vec![];
        buf.append(&mut task1.serialize()?);
        buf.append(&mut task2.serialize()?);

        // parse one by one
        let (result1, remainder) =
            Task::deserialize(&buf)?.ok_or("should parse task1".to_owned())?;
        assert_eq!(result1, task1);
        let (result2, remainder) =
            Task::deserialize(&remainder)?.ok_or("should parse task1".to_owned())?;
        assert_eq!(result2, task2);
        assert!(remainder.is_empty());

        // parse many
        let (results, remainder) =
            Task::deserialize_many(&buf)?.ok_or("should parse 2 tasks".to_owned())?;
        assert_eq!(results.to_vec(), vec![task1, task2]);
        assert!(remainder.is_empty());
        Ok(())
    }
}
