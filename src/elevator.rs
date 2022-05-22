/*
电梯对象
User case:
1.响应调导器的请求
2.开门
3.关门
4.去目标楼层
*/
use std::borrow::{Borrow, BorrowMut};
use std::future::Future;
use std::ops::{AddAssign, Deref};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use futures::StreamExt;
use wutil::convert::StaticRef;

use crate::task::{ElevatorTask, TaskType};

#[derive(PartialEq)]//使其可以用==比较
pub enum Status {
    Stop,
    goingUp,
    goingDown,
    Error,
}

pub struct Elevator {
    pub id: usize,
    //floor字段会用于多线程间共享数据，所以使用原子引用计数Arc<T>与互斥器Mutex
    pub floor: Arc<Mutex<i32>>,
    pub is_up: bool,
    pub capacity: i32,
    pub status: Status,
}

struct DoTaskFuture {
    ele: &'static Elevator,
    task: &'static ElevatorTask,
}

impl DoTaskFuture {
    fn new(ele: &'static Elevator, task: &'static ElevatorTask) -> Self {
        Self {
            ele,
            task,
        }
    }
}

impl Future for DoTaskFuture {
    type Output = i32;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let me = Pin::into_inner(self);
        //调用wake()来再次调用poll()
        cx.waker().wake_by_ref();
        let mut floor = me.ele.floor.lock().unwrap();
        if *floor < me.task.target_floor {
            *floor += 1;
            me.ele.up_floor(*floor);
            Poll::Pending
        } else if *floor > me.task.target_floor {
            *floor -= 1;
            me.ele.down_floor(*floor);
            Poll::Pending
        } else {
            //到达目标楼层
            me.ele.open_door(me.task.target_floor);
            Poll::Ready(0)
        }
    }
}

impl Elevator {
    pub fn new(id: usize, init_floor: Arc<Mutex<i32>>) -> Self {
        Self {
            id,
            floor: init_floor,
            is_up: false,
            capacity: 15,
            status: self::Status::Stop,
        }
    }

    //up_floor(self) if takes self as input, which means it consumes self,
    //so change the function signature to take a reference to self,
    pub fn up_floor(&self, floor: i32) {
        self.status == Status::goingUp;
        println!("Elevator:{:?} going up, current floor:{:?}", self.id, floor);
        sleep(Duration::from_millis(500));
    }

    pub fn down_floor(&self, floor: i32) {
        self.status == Status::goingDown;
        //println!("going down, current floor:{:?}", floor);
        println!("Elevator:{:?} going down, current floor:{:?}", self.id, floor);
        sleep(Duration::from_millis(500));
    }

    pub fn open_door(&self, floor: i32) {
        println!("Elevator:{:?} arrived in floor:{:?}, door opening", self.id, floor);
        sleep(Duration::from_millis(2000));
        self.close_door();
    }

    pub fn close_door(&self) {
        println!("Elevator:{:?} door close", self.id);
        self.status == Status::Stop;
    }

    pub fn do_job(self: &'static Elevator, task: &'static ElevatorTask) -> impl Future<Output=i32> {
        DoTaskFuture::new(self, task)
    }
}