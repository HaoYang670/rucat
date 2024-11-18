# Rucat

Unified gateway to create, connect and manage data engine on Kubernetes.

Rucat is a Boy / Girl name, meaning is Guider, Discipline and Adventurer. The Numerology Number for the name Rucat is 9.

## Architecture

### Idea

1. fully async
2. decouple rest server and k8s, apache spark

```mermaid
flowchart
    server(rucat server)
    spark(Apache Spark)
    monitor(rucat state monitor)
    k8s-api(k8s api server)
    db[(surreal db)]
    user(rucat client)
    user -- REST requests / RPC --> server

    subgraph k8s
    server -- create, remove, manage engine / get engine info --> db
    monitor -- regular engine state update --> db
    monitor -- create(delete) engine pod  / read pod info --> k8s-api
    k8s-api -- manage --> spark
    end
```

## Rucat Engine State

```mermaid
stateDiagram

    [*] --> Pending1: CREATE
    Pending1 --> Pending2: create k8s pod
    Pending1 --> Terminated: STOP
    Pending1 --> [*]: DELETE

    Pending2 --> Running: pod running
    Pending2 --> Terminating1: STOP
    Pending2 --> Deleting1: DELETE

    Running --> Terminating1: STOP
    Running --> Deleting1: DELETE

    Terminating1 --> Running: RESTART
    Terminating1 --> Terminating2: delete pod

    Terminating2 --> Terminated: pod removed

    Terminated --> Pending1: RESTART
    Terminated --> [*]: DELETE

    Deleting1 --> Deleting2: delete pod

    Deleting2 --> [*]: pod removed

    Error1 --> Error2: pod removed
    Error2 --> Pending1: RESTART
    Error2 --> [*]: DELETE

```

## How to test

```bash
bash test.sh
```

## TODO

1. test graceful shutdown
2. catch the spark driver log before deleting?
3. implement rucat-client (based on spark-connect-rs)
4. Test graceful shutdown <https://github.com/JosephLenton/axum-test/issues/88#issuecomment-2369720183>
5. Rewrite engine state using Surreal Literal type <https://surrealdb.com/docs/surrealql/datamodel/literals>
6. mock all, surrealdb and k8s. <https://github.com/asomers/mockall>
7. miri testing <https://github.com/rust-lang/miri>
8. fuzz testing <https://rust-fuzz.github.io/book/introduction.html>
9. shared spark v.s. exclusive spark (for example for batch job)
10. make all request fully async. tasks are submitted by storing info in cluster state, rucat monitor takes account of do the tasks and update the cluster state.
11. Arrow flight sql as protocol

## How to deploy on k8s

1. `helm install rucat rucat`
2. `kubectl port-forward <rucat server pod> 1234:3000`

## Debug

Dummy command that can make a pod running forever: `tail -f /dev/null`
