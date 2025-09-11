use bevy::prelude::*;


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
pub struct MainProgram {
    groups: Vec<WorkerGroup>,
    deletion_handler: Box<DeletionHandler>,
    image_data: MainImageData
}
impl MainProgram {
    pub fn init(image_data: &MainImageData, group_amount: u32) -> MainProgram {
        MainProgram {
            groups: (0..group_amount).map(|_| WorkerGroup::new(0, 0)).collect(),
            deletion_handler: Box::new(DeletionHandler::new()),
            image_data: image_data.clone()
        }
    }
}

pub struct WorkerGroup {
    x: u32,
    y: u32,
    workers: Vec<Worker>
}

impl WorkerGroup {
    pub fn new(start_x: u32, start_y: u32) -> WorkerGroup {
        WorkerGroup {
            x: start_x,
            y: start_y,
            workers: (0..4).map(|_| Worker::new()).collect()
        }
    }
}

pub struct Worker {

}

impl Worker {
    pub fn new() -> Worker {
        Worker {  }
    }
}

pub struct DeletionHandler {

}

impl DeletionHandler {
    pub fn new() -> DeletionHandler {
        DeletionHandler {  }
    }
}
