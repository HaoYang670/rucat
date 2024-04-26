use std::rc::Rc;

use super::{dependency::Dependency, partition::PartitionIndex, storage_level::StorageLevel};
use crate::{rdd::partition::Partition, task_context::TaskContext};
use rucat_common::error::{Result, RucatError};

/// Element types of RDD
#[derive(PartialEq, Debug)]
pub(super) enum RDDElem {
    U8(u8),
    Pair(Rc<RDDElem>, Rc<RDDElem>),
}

/// define RDD as  where RDDVariant is the enum of all kinds of RDDs.
/// Unlilke the RDD in Spark, we don't define the context as an argument of RDD as it is `global`
pub(super) struct RDD {
    dependencies: Vec<Dependency>,
    storage_level: Option<StorageLevel>,
    rdd_core: RDDVariant,
}

impl RDD {
    pub(super) fn new(
        dependencies: Vec<Dependency>,
        storage_level: Option<StorageLevel>,
        rdd_core: RDDVariant,
    ) -> Self {
        RDD {
            dependencies,
            storage_level,
            rdd_core,
        }
    }

    /// Return how this RDD depends on parent RDDs.
    fn get_dependencies(&self) -> &[Dependency] {
        &self.dependencies
    }

    /// the first parent RDD
    fn get_first_parent(&self) -> Option<&RDD> {
        self.dependencies.first().map(|d| d.get_rdd())
    }

    /// Return the set of partitions in the RDD.
    /// The partitions returned must be sorted by [Partition::index].
    /// WARNING: the function now is self-recursion with no termination.
    fn get_partitions(&self) -> &[Partition] {
        match &self.rdd_core {
            RDDVariant::MapPartitionRDD { .. } => self.get_first_parent().unwrap().get_partitions(),
        }
    }

    pub fn compute(&self, split: Partition, context: TaskContext) -> impl Iterator<Item = RDDElem> {
        match &self.rdd_core {
            RDDVariant::MapPartitionRDD { f } => f(
                context,
                split.index(),
                // the unwrap here is unsound
                Box::new(self.get_first_parent().unwrap().into_iter()), // what if dependency is empty?
            ),
        }
    }

    fn persist(mut self, level: StorageLevel) -> Result<Self> {
        match self.storage_level {
            Some(_) => RucatError::CannotChangeStorageLevelError.into(),
            None => {
                // TODO: persist the rdd data
                self.storage_level = Some(level);
                Ok(self)
            }
        }
    }
}

impl IntoIterator for &RDD {
    type Item = RDDElem;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        todo!()
    }
}

pub(super) enum RDDVariant {
    /// An RDD that applies the provided function to every partition of the parent RDD.
    MapPartitionRDD {
        f: Box<
            dyn Fn(
                TaskContext,
                PartitionIndex,
                Box<dyn Iterator<Item = RDDElem>>,
            ) -> Box<dyn Iterator<Item = RDDElem>>,
        >,
    },
}
