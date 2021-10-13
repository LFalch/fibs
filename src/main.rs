use std::collections::{BinaryHeap, VecDeque};
use std::env::args;
use std::io::{stdin, BufRead};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Condvar, Mutex,
};
use std::thread;

fn fib(n: u128) -> u128 {
    if n <= 1 {
        1
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

static RUN: AtomicBool = AtomicBool::new(true);

fn main() {
    let new_work = Arc::new((
        Condvar::new(),
        Mutex::new(BinaryHeap::<u128>::with_capacity(8)),
    ));

    let num_threads = args()
        .nth(1)
        .and_then(|arg| arg.trim().parse().ok())
        .unwrap_or_else(num_cpus::get);

    let threads: Vec<_> = {
        (0..num_threads)
            .map(|_| {
                let work = new_work.clone();
                thread::spawn(move || loop {
                    let mut lock = work
                        .0
                        .wait_while(work.1.lock().unwrap(), |queue| {
                            queue.is_empty() && RUN.load(Ordering::Relaxed)
                        })
                        .unwrap();

                    if let Some(n) = lock.pop() {
                        drop(lock);
                        println!("fib({}) = {}", n, fib(n));
                    } else {
                        if !RUN.load(Ordering::Acquire) {
                            break;
                        }
                    }
                })
            })
            .collect()
    };

    for line in stdin().lock().lines() {
        let line = line.unwrap();
        if let Ok(n) = line.parse::<u128>() {
            new_work.1.lock().unwrap().push(n);
            new_work.0.notify_one();
        }
    }

    RUN.store(false, Ordering::Release);

    new_work.0.notify_all();

    for thread in threads {
        thread.join().unwrap();
    }
}
