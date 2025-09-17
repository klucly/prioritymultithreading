use bevy::prelude::*;

use std::sync::RwLock;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::sync::mpsc;

use rand::Rng;
use libc::{sched_param, sched_setscheduler, SCHED_FIFO, getpid, pid_t};


#[derive(Resource, Clone)]
pub struct MainImageData {
    handle: Handle<Image>,
    width: u32,
    height: u32,
    data_ptr: usize,
}

impl MainImageData {
    pub fn new(handle: Handle<Image>, width: u32, height: u32, data_ptr: usize) -> MainImageData {
        MainImageData { handle, width, height, data_ptr }
    }
    pub fn handle(&self) -> Handle<Image> {self.handle.clone()}
    pub fn width(&self) -> u32 {self.width}
    pub fn height(&self) -> u32 {self.height}
    pub fn data_ptr(&self) -> usize {self.data_ptr}
    pub fn _set_data_ptr(&mut self, value: usize) {self.data_ptr = value}
}

#[derive(Resource)]
pub struct MainController {
    groups: Vec<WorkerGroup>,
    deletion_handler: Box<DeletionHandler>,
}
impl MainController {
    pub fn new(image_data: &MainImageData, group_amount: u32) -> MainController {
        let image_data = Arc::new(image_data.clone());
        MainController {
            groups: (0..group_amount).map(|_| WorkerGroup::new(image_data.clone(), 0, 0)).collect(),
            deletion_handler: Box::new(DeletionHandler::new()),
        }
    }
    pub fn init(&mut self) {
        (&mut self.groups).into_iter().for_each(|group| group.init());
    }
    pub fn start_all(&self) -> Result<(), std::sync::PoisonError<std::sync::RwLockWriteGuard<'_, WorkerStatus>>> {
        for group in &self.groups {
            group.start()?;
        }
        Ok(())
    }
    pub fn stop_all(&self) -> Result<(), std::sync::PoisonError<std::sync::RwLockWriteGuard<'_, WorkerStatus>>> {
        for group in &self.groups {
            group.stop()?;
        }
        Ok(())
    }
    pub fn update_priorities(&self, priorities: Vec<i32>) -> Result<(), String> {
        (&self.groups).iter().zip(priorities).try_for_each(|(group, priority)| group.set_priority(priority))?;
        Ok(())
    }
    
}

fn set_thread_priority(priority: i32, pid: pid_t) -> Result<(), String> {
    unsafe {
        let param = sched_param { sched_priority: priority };
        let result = sched_setscheduler(pid, SCHED_FIFO, &param);
        if result == -1 {
            return Err(format!("Failed to set priority {} for a thread with pid: {}", priority, pid).to_string());
        }
        println!("Set priority {} for a thread with pid: {}", priority, pid);
    }
    Ok(())
}

#[derive(Clone, Default, PartialEq, Eq)]
pub enum WorkerStatus {
    #[default]
    Idle,
    Running,
}

pub struct WorkerGroup {
    x: u32,
    y: u32,
    workers: Vec<Worker>,
    status: Arc<RwLock<WorkerStatus>>,
}

impl WorkerGroup {
    // TODO: disbanding groups and killing threads
    pub fn new(image_data: Arc<MainImageData>, start_x: u32, start_y: u32) -> WorkerGroup {
        let mut group = WorkerGroup {
            x: start_x,
            y: start_y,
            workers: Vec::new(),
            status: Arc::new(RwLock::new(WorkerStatus::default()))
        };
        group.workers = (0..4).map(|_| Worker::new(image_data.clone(), group.status.clone())).collect();
        group
    }
    pub fn init(&mut self) {
        (&mut self.workers).into_iter().for_each(|worker| worker.start());
    }
    pub fn start(&self) -> Result<(), std::sync::PoisonError<std::sync::RwLockWriteGuard<'_, WorkerStatus>>> {
        *self.status.write()? = WorkerStatus::Running;
        Ok(())
    }
    pub fn stop(&self) -> Result<(), std::sync::PoisonError<std::sync::RwLockWriteGuard<'_, WorkerStatus>>> {
        *self.status.write()? = WorkerStatus::Idle;
        Ok(())
    }
    pub fn set_priority(&self, priority: i32) -> Result<(), String>{
        (&self.workers).iter().try_for_each(|worker| worker.set_priority(priority))?;
        Ok(())
    }
}

pub struct Worker {
    image_data: Arc<MainImageData>,
    status: Arc<RwLock<WorkerStatus>>,
    worker_thread: Option<thread::JoinHandle<()>>,
    pid: i32
}

impl Worker {
    pub fn new(image_data: Arc<MainImageData>, status: Arc<RwLock<WorkerStatus>>) -> Worker {
        Worker { image_data, status, worker_thread: None, pid: 0 }
    }

    fn start(&mut self) {
        let image_data = self.image_data.clone();
        let status = self.status.clone();
        let color = random_color();
        let (tx, rx) = mpsc::channel();

        self.worker_thread = Some(thread::spawn(move || unsafe {Worker::handle(tx, image_data, status, color);}));
        self.pid = rx.recv().expect("Couldn't receive");
    }

    unsafe fn handle(tx: mpsc::Sender<i32>, image_data: Arc<MainImageData>, status: Arc<RwLock<WorkerStatus>>, color: (u8, u8, u8, u8)) {
        let img_ptr = image_data.data_ptr as *mut u8;
        tx.send(getpid()).expect("Couldn't get pid of a thread");
        loop {
            let status_read_attempt = status.read();
            let status = match status_read_attempt {
                Ok(a) => a,
                _ => {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
            };
            if *status == WorkerStatus::Idle {
                thread::sleep(Duration::from_millis(100));
                continue;
            }
            
            for y in 0..image_data.height {
                for x in 0..image_data.width {
                    let index = 4 * (x + y * image_data.width) as usize;
                    *img_ptr.add(index+0) = color.0;
                    *img_ptr.add(index+1) = color.1;
                    *img_ptr.add(index+2) = color.2;
                    *img_ptr.add(index+3) = color.3;
                }
            }
        }
    }
    fn set_priority(&self, priority: i32) -> Result<(), String> {
        set_thread_priority(priority, self.pid)?;
        Ok(())
    }
}

pub struct DeletionHandler {

}

impl DeletionHandler {
    pub fn new() -> DeletionHandler {
        DeletionHandler {  }
    }
}

pub fn random_color() -> (u8, u8, u8, u8) {
    let mut rng = rand::rng();
    (
        rng.random_range(0..=255),
        rng.random_range(0..=255),
        rng.random_range(0..=255),
        255
    )
}