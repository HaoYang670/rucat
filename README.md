# Rucat

Unified gateway to create, connect and manage data engine on any platform.

Rucat name meaning is Guider, Discipline, Adventurer and Rucat is a Boy / Girl name. The Numerology Number for the name Rucat is 9.

```mermaid
flowchart
    server(rucat server)
    engine(rucat engine)
    spark(Apache Spark)
    monitor(rucat monitor)
    db[(surreal db)]
    user -- http requests --> server
    monitor -- regular healthy check --> db
    server -- read / write engine info --> db
    server -- create / stop / restart --> engine
    engine -- update engine info --> db
    engine -- create / connect --> spark
```

## Rucat Engine State

```mermaid
stateDiagram
    [*] --> Pending
    Pending --> Running
    Running --> Stopped
    Stopped --> Pending
    Stopped --> [*]
```

## How to test

```bash
bash test.sh
```

## How to run

```bash
bash ./example/run.sh
```

## TODO

1. Add engine discovery time
2. Add heartbeat for rucat engine -> rucat engine update the discovery time in database regularly.
3. Implement rucat monitor to check the database regularly and detect unhealthy engines
4. server sends requests to engine by RPC.
5. implement spark submit. (standalone, local mode for now, k8s mode in the future)
6. mock rucat engine for testing / embedded rucat engine ?
7. Add connect engine function to connect to the engine that is not created by rucat. (by spark-connect-rs for instance)
8. 3 mode for rucat server:
  embedded mode: use in memory db, can only create engine in the same process (embedded)
  local mode: use local mode db, can create engines embedded or locally
  remote mode: use remote db, can create engines embedded, locally or remotely.
