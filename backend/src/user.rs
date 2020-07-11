use crate::robot::Robot;

#[derive(Clone)]
pub struct User {
    robot: Robot,
}

impl User {
    pub fn new() -> Self {
        let robot = Robot::default();
        Self { robot }
    }
}
