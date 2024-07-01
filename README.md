# Rucat
Rebuild distributed system in Rust, with category theory in mind.

Rucat name meaning is Guider, Discipline, Adventurer and Rucat is a Boy / Girl name. The Numerology Number for the name Rucat is 9. 

Client --- `Task` ---> `Cluster Manager` 
         --- create / delete / workload control / send `Task` ---> `Driver`
             --- create / delete / workload control / send `SubTask` ---> `Worker`

Executors are servers of Drivers, and Drivers are servers of Cluster Manager.
Finally, Cluster Manager is a server to receive users (client) requests.

## Client

Users should use functions provided by Recat to create their Task. Then compile and run the client, the Task
will be sent to Cluster Manager which will return the result then.

## Cluster Manager

### Cluster management

### Task assignement


## Driver

### Workers management

Collect metrics from `Worker` to infer the pressure. `Cluster` should always try to make load balance on Workers.
If all workers have too much pressure, then `Cluster` should create new `Worker`s to reduce the pressure.

Deleting workers should be careful. It is not a good practice to delete and create workers back and force. 
I prefer deleting workers passively (lazily), which means do not delete a worker until we have to (resource limitation, for example).

### Execute Task

Split task to several subtasks;
send request to workers and collect the subresult back;
merge subresult to the final result.

### Pressure management

## Worker

execute the subtask sent to it, and return the result back.

### Pressure management

## Jobs

### Job

`Job` a type for users (client) to define their workload. `Job`s are sent from client to cluster manager and the manager will assign
each `Job` to a cluster.
`Task` -> (`Stage`1 -> `Stage2` -> ... `Stage n`)
           /  |  \
      Task1 Task2 Task3

#### Multi Stage Task (TODO)


`Task` can have multi stages (multi map-reduce stage).
The `Client`(or `Cluster Manager`, or `Driver`) should have functions to convert users' workload to multi-stages.


### SubTask

Cluster driver will split the `Task` into several `subTasks` after receiving it from the cluster manager. Each `SubTask` will be sent to a `Worker` to execute.

## Lifetime

`Client`: dynamic lifetime

`Cluster Manager` >= `Driver` >= `Worker`

## TODO


replace Clap with Json config file to support enum
use cargo-udeps to detect unused dependencies
surrealdb, support local process mode for rucat engine to connect
unique port for each rucat engine (how does rucat server know the port of engine? write into the db?)
mock rucat engine for testing / embedded rucat engine ? 
profile rust
3 mode for rucat server:
  embedded mode: use in memory db, can only create engine in the same process (embedded)
  local mode: use local mode db, can create enines embeddly or locally
  remote mode: use remote db, can create engines embedded, locally or remotely.