use std::borrow::Borrow;
use std::collections::LinkedList;
use std::sync::{Arc, mpsc, Mutex};
use std::sync::atomic::AtomicBool;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

//使用crossbeam::channel::Sender可以定义T
use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;
use crossbeam::channel::unbounded;
use crossbeam_utils::sync::Parker;
use work_queue::Queue;
use wutil::convert::make_mut;
use wutil::convert::StaticRef;
use wutil::convert::StaticRefArray;
use wutil::static_refs;

use crate::elevator::Elevator;
use crate::scheduler::{Scheduler, SchedulerBuilder};
use crate::task::ElevatorTask;
use crate::task::TaskType;

mod elevator;
mod scheduler;
mod task;
mod task_handler;

const ELEVATORS_SIZE: usize = 5;

/*
使用生产者-消费者模式处理生成任务和处理任务
*/
fn main() {
    let threads = 5;
    static_refs! {
        global_queue = Queue::new(threads as usize, 16);
        msg_done = AtomicBool::new(false);
        unparkers = Vec::with_capacity(threads);
        elevators = Vec::with_capacity(ELEVATORS_SIZE);
    }
    //init elevator
    unsafe { make_mut(elevators) }.push(Elevator::new(0,Arc::new(Mutex::new(0))));
    unsafe { make_mut(elevators) }.push(Elevator::new(1,Arc::new(Mutex::new(1))));
    unsafe { make_mut(elevators) }.push(Elevator::new(2,Arc::new(Mutex::new(1))));
    unsafe { make_mut(elevators) }.push(Elevator::new(3,Arc::new(Mutex::new(0))));
    unsafe { make_mut(elevators) }.push(Elevator::new(4,Arc::new(Mutex::new(3))));
    // make parkers
    let parkers = StaticRefArray::new(threads as usize, || {
        let parker = Parker::new();
        let unparker = parker.unparker().clone();
        unsafe { make_mut(unparkers) }.push(unparker.clone());
        (parker, unparker)
    });
    // Safety: The life of it go along with the scheduler
    let parkers = unsafe { (&parkers).static_ref() };
    let mut s = SchedulerBuilder::new().build(threads, global_queue, parkers, msg_done, unparkers, elevators);
    let task1 = ElevatorTask::new(true, 2, TaskType::goingUp);
    let task2 = ElevatorTask::new(true, 10, TaskType::goingUp);
    let task3 = ElevatorTask::new(true, 15, TaskType::goingUp);
    let task4 = ElevatorTask::new(false, 18, TaskType::goingUp);
    let task5 = ElevatorTask::new(false, 1, TaskType::goingDown);
    s.execute(task1);
    s.execute(task2);
    //s.execute(task3);
    //s.execute(task4);
    //s.execute(task5);
    s.run();
}
