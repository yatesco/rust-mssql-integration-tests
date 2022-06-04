An example of how to run database (MSSQL) integration tests.

The *key* point is that the container is lifecycled to the result from `docker.run` and the container is dropped when
that goes out of scope.

This is why `spin_up_database` returns the `node` that represents the container. If it didn't then the `node` would
go out of scope at the end of `spin_up_database`, the `drop` function would be called and the database container would
be stopped.

Also note: this runs the tests sequentially because each MSSQL database container is recommended at 4GB of RAM.

WARNING: this is all "prototype" code (i.e. hacky, here be dragons)

To run: `cargo test -- --nocapture`