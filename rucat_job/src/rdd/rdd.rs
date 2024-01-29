use super::{dependency::Dependency, partition::PartitionIndex, storage_level::StorageLevel};
use crate::{rdd::partition::Partition, task_context::TaskContext};

/// Element types of RDD
pub(super) enum RDDElem {
    U8(u8),
}

/// define RDD as (Dependencies, RDDVariant) where
/// RDDVariant is the enum of all kinds of RDDs.
/// Unlilke the RDD in Spark, we don't define the context as an argument of RDD as it is `global`
pub(super) struct RDD {
    dependencies: Vec<Dependency>,
    storage_level: Option<StorageLevel>,
    rdd_core: RDDVariant,
}

impl RDD {
    fn get_dependencies(&self) -> &Vec<Dependency> {
        &self.dependencies
    }

    /// the first parent RDD
    fn get_first_parent(&self) -> Option<&RDD> {
        self.dependencies.first().map(|d| d.get_rdd())
    }

    fn compute(&self, split: Partition, context: TaskContext) -> impl Iterator<Item = RDDElem> {
        match &self.rdd_core {
            RDDVariant::MapPartitionRDD { f } => f(
                context,
                split.index(),
                Box::new(self.get_first_parent().unwrap().into_iter()),
            ),
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

enum RDDVariant {
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
