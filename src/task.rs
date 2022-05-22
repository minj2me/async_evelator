/*
任务对象

User case:
1.向上走
2.向下走
3.要开门
4.要关门
5.停止运行
*/
use std::cmp::Ordering;

#[derive(Debug, PartialEq)]
pub enum TaskType {
    goingUp,
    goingDown,
    openDoor,
    closeDoor,
    Stop,
}

/*
电梯请求结构
*/
#[derive(Debug)]
pub struct ElevatorTask {
    //是否来自电梯内部的请求
    pub indoor_request: bool,
    pub target_floor: i32,
    //目标楼层
    pub task_status: TaskType,
}

impl ElevatorTask {
    pub fn new(indoor_request: bool, target_floor: i32, ts: TaskType) -> Self {
        Self {
            indoor_request,
            target_floor,
            task_status: ts,
        }
    }
}

//实现了PartialEq才能实现PartialOrd
impl PartialEq for ElevatorTask {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl Eq for ElevatorTask {
}

impl PartialOrd for ElevatorTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.target_floor.cmp(&other.target_floor))//以target_floor来排序
        //Some(other.target_floor.cmp(&self.target_floor))//
    }
}

//如要放在priority queue，就要impl Ord
impl Ord for ElevatorTask {
    fn cmp(&self, other: &Self) -> Ordering {
        self.target_floor.cmp(&other.target_floor)//以target_floor来排序
        //other.target_floor.cmp(&self.target_floor)//
    }
}