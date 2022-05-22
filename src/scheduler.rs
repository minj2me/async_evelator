/**
用于任务调度
*/
use std::borrow::Borrow;
use std::collections::{BinaryHeap, LinkedList, VecDeque};
use std::i32::MAX;
use std::ptr::{null, null_mut};
use std::sync::atomic::AtomicBool;
use std::thread;
use std::thread::JoinHandle;

use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;
use crossbeam::channel::unbounded;
use crossbeam::sync::Parker;
use crossbeam::sync::Unparker;
use work_queue::Queue;
use wutil::convert::StaticRef;
use wutil::convert::StaticRefArray;

use crate::elevator::Elevator;
use crate::elevator::Status;
use crate::scheduler;
use crate::task::{ElevatorTask, TaskType};
use crate::task_handler::handler_run;

pub struct Scheduler {
    workers: Vec<Worker>,
    task_queue: &'static Queue<ElevatorTask>,
}

pub struct Worker {
    handle: JoinHandle<()>,
    unparker: Unparker,
}

pub struct SchedulerBuilder;

impl SchedulerBuilder {
    pub fn new() -> Self {
        Self
    }
    //使用&'static是因为不知道线程会执行多久，因此要与app生命周期同一时长
    pub fn build(
        self,
        threads: usize,
        global_queue: &'static Queue<ElevatorTask>,
        parkers: &'static StaticRefArray<(Parker, Unparker)>,
        msg_done: &'static AtomicBool,
        unparkers: &'static Vec<Unparker>,
        elevators: &'static Vec<Elevator>,
    ) -> Scheduler {
        let mut workers = Vec::with_capacity(threads as usize);
        let mut local_queues = global_queue.local_queues();
        let mut parkers_iter = parkers.iter();
        let (et_tx, et_rx) = unbounded();
        //多线程的异步通道
        for id in 0..threads {
            let rx = et_rx.clone();
            thread::spawn(move || handler_run(rx, global_queue, unparkers, elevators));
        }

        for id in 0..threads {
            let tx = et_tx.clone();
            let mut local_queue = local_queues.next().unwrap();
            let (parker, unparker) = parkers_iter.next().unwrap();
            let h = thread::spawn(move || {
                let mut priority_queue = BinaryHeap::new();
                loop {
                    while let Some(task) = local_queue.pop() {
                        priority_queue.push(task);
                    }
                    if priority_queue.is_empty() {
                        //blocks the current thread until the token is made available.
                        //Wakes up when `unpark()` provides the token.
                        parker.park();
                    } else {
                        while let Some(task) = priority_queue.pop() {
                            if task.task_status == TaskType::Stop {
                                if priority_queue.is_empty() {
                                    //println!("ALL TASK DONE.");
                                }
                                return;
                            }
                            tx.send(task);
                        }
                    }
                }
            });
            workers.push(Worker {
                handle: h,
                unparker: unparker.clone(),
            });
        }

        drop(et_tx);

        Scheduler {
            workers,
            task_queue: global_queue,
        }
    }
}

impl Scheduler {
    pub fn execute(&mut self, task: ElevatorTask) {
        self.task_queue.push(task);
    }

    pub fn join(self) {
        self.notify_all();
    }

    pub fn notify_all(&self) {
        for w in &self.workers {
            w.unparker.unpark();
        }
    }

    pub fn run(self) {
        for w in self.workers {
            w.handle.join().unwrap();
        }
    }
}