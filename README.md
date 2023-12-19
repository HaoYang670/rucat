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

## Worker

execute the subtask sent to it, and return the result back.

## Task



## Lifetime

`Client`: dynamic lifetime

`Cluster Manager` >= `Driver` >= `Worker`