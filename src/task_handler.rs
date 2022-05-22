/*
用于接收任务处理，分排任务给电梯

User case:
1.接收外部请求:
2.选择合适的elevator去处理请求
3.返回处理结果
*/
use std::borrow::{Borrow, BorrowMut};
use std::i32::MAX;
use std::io;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::Duration;

use crossbeam::channel::Receiver;
use crossbeam::sync::Unparker;
use work_queue::Queue;
use wutil::convert::StaticRef;
use wutil::random::gen;

use crate::elevator::{Elevator, Status};
use crate::task::{ElevatorTask, TaskType};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::thread::sleep;

pub fn handler_run(
    rx: Receiver<ElevatorTask>,
    ready_queue: &'static Queue<ElevatorTask>,
    unparkers: &'static Vec<Unparker>,
    elevators: &'static Vec<Elevator>,
) {
    futures::executor::block_on(async {
        for mut task in rx {
            println!("###> received task to floor:{:?}", task.target_floor);
            let mut cur = 0;
            let mut value = MAX;
            let mut do_task_ele = elevators.get(0).unwrap();
            for ele in elevators {
                cur = cal_weight(ele, task.borrow());
                if cur < value {
                    value = cur;
                    do_task_ele = ele;
                }
            }
            println!("===> handle task to floor:{:?} is elevator:{:?}, job start", task.target_floor, do_task_ele.id);
            //do_job为异步执行方法，当future返回Poll::Ready时，表示任务远成，才继续执行do_job()后面的代码
            do_task_ele.do_job(unsafe { (&task).static_ref() }).await;
            println!("***> handle task to floor:{:?} is elevator:{:?}, job DONE", task.target_floor, do_task_ele.id);
            task.task_status = TaskType::Stop;
            //处理完后，将task put back to ready_queue，并通过unparkers的unpark 去 Wakes up Scheduler来检查结果.
            ready_queue.push(task);
            let maybe_awaken = gen(0..unparkers.len());
            unsafe {
                unparkers.get_unchecked(maybe_awaken).unpark();
            }
        }
    });
}

/*
计算各电梯接受调度的权值，返回值越小，表示此电梯越合适
*/
fn cal_weight(elevator: &Elevator, task: &ElevatorTask) -> i32 {
    //计算出的值越小，表示此电梯响应此调度请求越合适
    if elevator.status == Status::Error {
        return MAX;
    }
    let mut floor = elevator.floor.lock().unwrap();
    //如果电梯空闲的，计算ta与目标的距离
    if elevator.status == Status::Stop {
        return (*floor - task.target_floor).abs();
    }
    //判断电梯是否顺路
    if task.target_floor >= *floor && elevator.status == Status::goingUp {
        return task.target_floor - *floor;
    }
    if task.target_floor <= *floor && elevator.status == Status::goingDown {
        return *floor - task.target_floor;
    }
    MAX - 1
}