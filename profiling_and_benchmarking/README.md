# Profiling and benchmarking README

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
