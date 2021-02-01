// Ref: https://github.com/richardanaya/executor/blob/master/executor/examples/helloworld/src/main.rs
use core::future::Future;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
struct Foo {}

impl Future for Foo {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(())
    }
}

fn a() -> impl Future<Output = ()> {
    println!("[a] hello world");
    Foo {}
}

fn b() -> impl Future<Output = ()> {
    println!("[b] hello world");
    Foo {}
}

fn main() -> () {
    async_rt::run(async {
        b().await;
        a().await;
    });
    async_rt::block(async {
        a().await;
    });
}
