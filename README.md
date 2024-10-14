# Rucat

Unified gateway to create, connect and manage data engine on any platform.

Rucat name meaning is Guider, Discipline, Adventurer and Rucat is a Boy / Girl name. The Numerology Number for the name Rucat is 9.

```mermaid
flowchart
    server(rucat server)
    spark(Apache Spark)
    monitor(rucat monitor)
    db[(surreal db)]
    user -- REST requests --> server
    monitor -- regular healthy check --> db
    server -- read / write engine info --> db
    server -- create / stop / restart --> spark
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

1. test graceful shutdown
2. redesign engine state: depends on pod state
3. catch the spark driver log before deleting?
4. implement rucat-client (based on spark-connect-rs)
5. deploy surreal on k8s
6. Test graceful shutdown <https://github.com/JosephLenton/axum-test/issues/88#issuecomment-2369720183>
7. Rewrite engine state using Surreal Literal type <https://surrealdb.com/docs/surrealql/datamodel/literals>

## How to deploy on k8s

1. `helm install rucat rucat`
2. `kubectl port-forward <rucat server pod> 1234:3000`

## How to submit spark

1. `kubectl create clusterrolebinding rucat-role --clusterrole=edit --serviceaccount=default:rucat-server --namespace=default`
2. go into the rucat server pod
3. install java: `apt install openjdk-17-jdk`, `export JAVA_HOME=/usr/lib/jvm/java-17-openjdk-arm64/`
4. download spark 3.5.3 (only the `/sbin` is useful I think ): `wget https://dlcdn.apache.org/spark/spark-3.5.3/spark-3.5.3-bin-hadoop3.tgz`, `tar -xvzf spark-3.5.3-bin-hadoop3.tgz`

## Debug

Dummy command that can make a pod running forever: `tail -f /dev/null`
