# Ordinator
Ordinator is a multi-agent scheduling system created in Rust. The system is revolved around agents
that each schedule a specific part of the scheduling process in real-time and then communicates 
their solutions to each other and to the user of the system in the form of RESTful API endpoints.

The real-time responsiveness of the systems means that each agent in the scheduling process will be 
able to react to incoming information from the system whenever and whereever it arrives in the 
scheduling process.


## Imperium 
Imperium is the command line interface to the Ordinator scheduling systems. It contains all the ways
that users should be able to interact with the system. 


## Tracing 
Tracing is a crucial aspect of understand the code. The log level can be set dynamically using 
Imperium.


# Profiling and benchmarking

This file is to explain and tell how to profile and benchmark the scheduling system.

## Profiling 
Profiling is done throught the tracing.rs and tracing-flame.rs packages. By adding the 

```
#[instrument] 
fn fun(par: Par) {
    // Do some calculation
}
```

This can lead to serious performance issues if the `par` argument is large and nested, as the 
instrument macro also applies tracing/logging to the function arguments. In that case one should 
use `#[instrument(skip(par))]` on the function definition.


## Benchmarking
