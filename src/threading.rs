use std::thread;
use std::time::Duration;
use winapi::um::processthreadsapi::{GetCurrentThread, SetThreadPriority};
use winapi::um::winbase::{
    THREAD_PRIORITY_LOWEST, THREAD_PRIORITY_BELOW_NORMAL, THREAD_PRIORITY_NORMAL,
    THREAD_PRIORITY_ABOVE_NORMAL, THREAD_PRIORITY_HIGHEST,
};

fn set_priority(priority: i32) {
    unsafe {
        let handle = GetCurrentThread();
        if SetThreadPriority(handle, priority) == 0 {
            eprintln!("Failed to set priority {priority}");
        }
    }
}

fn worker(id: usize, priority: i32) {
    set_priority(priority);

    for i in 0..1_000_000_000 {
        if i % 100_000_000 != 0 {
            continue;
        }
        println!("Thread {id} (prio {priority}) -> {i}");
        thread::sleep(Duration::from_millis(50));
    }
}

fn main() {
    let priorities = [
        THREAD_PRIORITY_LOWEST,
        THREAD_PRIORITY_BELOW_NORMAL,
        THREAD_PRIORITY_NORMAL,
        THREAD_PRIORITY_ABOVE_NORMAL,
        THREAD_PRIORITY_HIGHEST,
    ];

    let mut handles = vec![];

    for (i, &prio) in priorities.iter().enumerate() {
        handles.push(thread::spawn(move || worker(i, prio as i32)));
    }

    for h in handles {
        h.join().unwrap();
    }
}
