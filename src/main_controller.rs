use bevy::log;
use bevy::prelude::*;

use std::sync::RwLock;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::sync::mpsc;

use rand::Rng;

use windows::Win32::System::Threading::{
    GetCurrentThreadId, OpenThread, SetThreadPriority,
    THREAD_SET_INFORMATION, THREAD_PRIORITY
};
use windows::core;

#[derive(Resource, Clone)]
pub struct MainImageData {
    handle: Handle<Image>,
    width: i32,
    height: i32,
    data_ptr: usize,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Color(u8, u8, u8, u8);

impl MainImageData {
    pub fn new(handle: Handle<Image>, width: i32, height: i32, data_ptr: usize) -> MainImageData {
        MainImageData { handle, width, height, data_ptr }
    }
    pub fn handle(&self) -> Handle<Image> {self.handle.clone()}
    pub fn width(&self) -> i32 {self.width}
    pub fn height(&self) -> i32 {self.height}
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
            groups: (0..group_amount).map(|_| WorkerGroup::new(image_data.clone())).collect(),
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
    pub fn update_priorities(&self, priorities: Vec<i32>) -> core::Result<()> {
        (&self.groups).iter().zip(priorities).try_for_each(|(group, priority)| group.set_priority(priority))?;
        Ok(())
    }
    
}

fn set_thread_priority(priority: i32, pid: u32) -> core::Result<()> {
    log::info!("Set {} to priority {}", pid, priority);
    let thread_handle = unsafe { OpenThread(THREAD_SET_INFORMATION, false, pid)? };
    unsafe { SetThreadPriority(thread_handle, THREAD_PRIORITY(priority))? };
    Ok(())
}

#[derive(Clone, Default, PartialEq, Eq)]
pub enum WorkerStatus {
    #[default]
    Idle,
    Running,
}

pub struct WorkerGroup {
    workers: Vec<Worker>,
    status: Arc<RwLock<WorkerStatus>>,
}

impl WorkerGroup {
    // TODO: disbanding groups and killing threads
    pub fn new(image_data: Arc<MainImageData>) -> WorkerGroup {
        let mut group = WorkerGroup {
            workers: Vec::new(),
            status: Arc::new(RwLock::new(WorkerStatus::default()))
        };
        group.workers = (0..4).map(|_| Worker::new(image_data.clone(), group.status.clone())).collect();
        group
    }
    pub fn init(&mut self) {
        let color = random_color();
        log::info!("{:?}", color);
        let pos = self.workers[0].get_random_pos();
        unsafe { self.workers[0].set_color(pos[0], pos[1], color) };
        (&mut self.workers).into_iter().for_each(|worker| {worker.spawn(color)});
    }
    pub fn start(&self) -> Result<(), std::sync::PoisonError<std::sync::RwLockWriteGuard<'_, WorkerStatus>>> {
        *self.status.write()? = WorkerStatus::Running;
        Ok(())
    }
    pub fn stop(&self) -> Result<(), std::sync::PoisonError<std::sync::RwLockWriteGuard<'_, WorkerStatus>>> {
        *self.status.write()? = WorkerStatus::Idle;
        Ok(())
    }
    pub fn set_priority(&self, priority: i32) -> core::Result<()>{
        (&self.workers).iter().try_for_each(|worker| worker.set_priority(priority))?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct Worker {
    image_data: Arc<MainImageData>,
    status: Arc<RwLock<WorkerStatus>>,
    worker_thread: Option<Arc<thread::JoinHandle<()>>>,
    pid: u32
}

impl Worker {
    pub fn new(image_data: Arc<MainImageData>, status: Arc<RwLock<WorkerStatus>>) -> Worker {
        Worker { image_data, status, worker_thread: None, pid: 0 }
    }

    fn spawn(&mut self, color: Color) {
        let (tx, rx) = mpsc::channel();
        let other_thread_self = self.clone();

        self.worker_thread = Some(Arc::new(thread::spawn(move || unsafe {other_thread_self.handle(tx, color)})));
        self.pid = rx.recv().expect("Couldn't receive");
    }

    unsafe fn handle(self, tx: mpsc::Sender<u32>, color: Color) {
        tx.send(unsafe {GetCurrentThreadId()} ).expect("Couldn't get pid of a thread");

        self.wait_for_ready();
        
        let mut pos: [i32; 2];

        for i in 0.. {
            if i % 1000 == 0 { self.wait_for_ready() }

            pos = self.get_random_pos();

            for _ in 0..1000 {
                self.move_random_direction(&mut pos);
                let stop = unsafe { self.fill_if_near_neighbor(&pos, color) };
                if stop { break }
            }
        }
    }

    fn get_random_pos(&self) -> [i32; 2] {
        [
            rand::random_range(0..self.image_data.width()),
            rand::random_range(0..self.image_data.height())
        ]
    }

    fn set_priority(&self, priority: i32) -> core::Result<()> {
        set_thread_priority(priority, self.pid)?;
        Ok(())
    }

    fn move_random_direction(&self, pos: &mut [i32; 2]) {
        let direction = rand::random_range(0..4);
        let (x_dir, y_dir) = match direction {
            0 => ( 1,  0),
            1 => (-1,  0),
            2 => ( 0,  1),
            3 => ( 0, -1),
            _ => panic!("Random range for a step direction has been setup wrongfully")
        }; 
        (pos[0], pos[1]) = (pos[0] + x_dir, pos[1] + y_dir);
    }

    fn hit_wall(&self, pos: &[i32; 2]) -> bool {
        pos[0] == self.image_data.width() - 1 ||
        pos[0] == 1 ||
        pos[1] == self.image_data.height() - 1 ||
        pos[1] == 1
    }

    unsafe fn fill_if_near_neighbor(&self, pos: &[i32; 2], color: Color) -> bool {
        for (x_bias, y_bias) in [(1,0), (-1,0), (0,1), (0,-1)] {

            let neighboring_color = unsafe { self.get_color(pos[0] + x_bias, pos[1] + y_bias) };
            if neighboring_color == color {
                unsafe { self.set_color(pos[0], pos[1], color) };
                return true;
            }
        }
        false
    }

    fn truncate_pos(&self, pos: &[i32; 2]) -> [i32; 2] {
        let mut new_pos = [0, 0];
        let (a, b) = (pos[0] - 1, (self.image_data.width() - 1));
        new_pos[0] = ((a % b) + b) % b;
        let (a, b) = (pos[1] - 1, (self.image_data.height() - 1));
        new_pos[1] = ((a % b) + b) % b;
        
        new_pos
    }

    fn wait_for_ready(&self) {
        loop {
            // If we run this function too fast,
            // it will die and never give permission to status
            // Wait a little to avoid that
            thread::sleep(Duration::from_nanos(10));

            let status_read_attempt = self.status.read();

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
            return;
        }
    }

    unsafe fn get_color(&self, x: i32, y: i32) -> Color {
        let pos = self.truncate_pos(&[x, y]);
        unsafe { self.get_color_from_index(self.get_index(pos[0], pos[1])) }
    }

    unsafe fn set_color(&self, x: i32, y: i32, color: Color) {
        let pos = self.truncate_pos(&[x, y]);
        unsafe { self.set_color_from_index(self.get_index(pos[0], pos[1]), color) }
    }

    unsafe fn get_color_from_index(&self, index: usize) -> Color {
        let img_ptr = self.image_data.data_ptr as *mut u8;
        
        unsafe {Color(
            *img_ptr.add(index+0),
            *img_ptr.add(index+1),
            *img_ptr.add(index+2),
            *img_ptr.add(index+3)
        )}
    }

    unsafe fn set_color_from_index(&self, index: usize, color: Color) {
        let img_ptr = self.image_data.data_ptr as *mut u8;

        unsafe {
            *img_ptr.add(index+0) = color.0;
            *img_ptr.add(index+1) = color.1;
            *img_ptr.add(index+2) = color.2;
            *img_ptr.add(index+3) = color.3;
        }
    }

    fn get_index(&self, x: i32, y: i32) -> usize {
        4 * (x as usize + y as usize * self.image_data.width() as usize)
    }
}


pub struct DeletionHandler {

}

impl DeletionHandler {
    pub fn new() -> DeletionHandler {
        DeletionHandler {  }
    }
}

pub fn random_color() -> Color {
    let mut rng = rand::rng();
    Color(
        rng.random_range(0..=255),
        rng.random_range(0..=255),
        rng.random_range(0..=255),
        255
    )
}