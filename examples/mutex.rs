use std::{sync::{Mutex, MutexGuard}, time::Duration};

use tokio::time::sleep;

async fn some_heavy_work() {
    sleep(Duration::from_secs(2)).await;


}

/// explanation
/// Send is an auto-trait that tells the compiler that this type is safe to be moves across threads
/// most types are Send, however there are types such as MutexGuard, Rc, RefCell etc which are not
/// To spawn a task in tokio, all variables must be type Send
/// however, there are cases like this:
/// 
async fn increment_and_do_stuff(mutex: &Mutex<i32>) {
    {
        let mut lock: MutexGuard<i32> = mutex.lock().unwrap(); // MutexGuard is not send, so it must be dropped before the thead reaches the await
        *lock += 1; 
    } // the lock MUST go out of scope before the next await

    some_heavy_work().await; // the task pauses here and may be resumed in another thread when its schedule back
    // since it may be resolved in another thread, everything till here must be Send type so that the compiler 
    // knows that its safe to move across threads. since MutexGuard is not Send, we must drop it before the await call
    // NOTE : in some mutex crates, their MutexGuards are Send, which means it will compile fine, but THERE STILL the chance
    // of occuring deadlocks
}

#[tokio::main]
async fn main() {

    let  mutex = Mutex::new(0);

    let _ = tokio::spawn( async move {
        increment_and_do_stuff(&mutex).await;
    });
}