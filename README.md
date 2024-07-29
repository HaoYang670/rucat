# Rucat

Unified gateway to create, connect and manage data engine on any platform.

Rucat name meaning is Guider, Discipline, Adventurer and Rucat is a Boy / Girl name. The Numerology Number for the name Rucat is 9.

## TODO

replace im memory database with any local database that can be created and connect in rust. (alternatively use command line in rucat server to create such a db, and delete it in graceful shutdown)
how does surrealdb start the web server <https://github.com/surrealdb/surrealdb/blob/07610d94119154ea922df92bbde759bbc2012533/src/net/mod.rs#L200>
mock rucat engine for testing / embedded rucat engine ?
rucat engine update engine state in database? Is it really needed?
Add connect engine function to connect to the engine that is not created by rucat. (by spark-connect-rs for instance)
3 mode for rucat server:
  embedded mode: use in memory db, can only create engine in the same process (embedded)
  local mode: use local mode db, can create engines embedded or locally
  remote mode: use remote db, can create engines embedded, locally or remotely.
