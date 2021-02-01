# async-rt
A Simple(Toy) Implementation of Rust Async Runtime.  
Just for Rust Async learning.  
Ref: [executor](https://github.com/richardanaya/executor)  

## example
```Rust
fn main() {
    // block mode
    async_rt::block(async {
        console_log("Hello world");
    })
}
```
or:  
```Rust
fn main() {
    // non-blocking mode
    async_rt::run(async {
        console_log("Hello world");
    })
}
```
More examples see [example](https://github.com/SKTT1Ryze/async-rt/example)  

## TODO
+ Add `async-rt-macros`
+ Add embedded example
+ ...