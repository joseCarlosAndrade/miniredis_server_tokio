//! solving the problems listed in mutex.rs
//! 
//! the safes way to deal with mutex is wrapping them in a struct 
//! and implementing non-async methods to lock it

use tokio::time::sleep;
use std::{sync::{Mutex, MutexGuard}, time::Duration};

struct CanIncrement {
    mutex: Mutex<i32>,
}


impl CanIncrement {
    fn increment(&self) { // sync here, not async
        let mut lock = self.mutex.lock().unwrap();
        *lock += 1;
    } // makes the lock go out of score safely and prevents deadlocks even if the Mutex implements the Send

    // do other sync funcions that must lock the mutex
    // ...
}

async fn increment_and_do_stuff(can_incre: &CanIncrement) {
    can_incre.increment();

    some_heavy_work().await;

}

async fn some_heavy_work() {
    sleep(Duration::from_secs(2)).await;


}

#[tokio::main]
async fn main() {

    let  mutex = CanIncrement { mutex: Mutex::new(0) };

    let _ = tokio::spawn( async move {
        increment_and_do_stuff(&mutex).await;
    });

}