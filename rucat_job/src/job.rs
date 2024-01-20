/// Job is a recursive type that contains stages
/// A job can be shared across different jobs as their component, just like a node in the linked list.
/// We can gain performance improvement from this, for example by caching the job result.
/// Job should be serialized because we need to send jobs from client to cluster manager
/// and cluster manager sends jobs to clusters (drivers) .
enum Job {
    Empty,
}
