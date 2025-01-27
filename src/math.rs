#[derive(Debug, Clone)]
pub enum Operation {
    PLUS,
    MINUS,
    MULTIPLY,
    DIVIDE
}

impl Operation {
    pub fn to_int(&self) -> i32 {
        match self {
            Operation::PLUS => 0,
            Operation::MINUS => 1,
            Operation::MULTIPLY => 2,
            Operation::DIVIDE => 3
        }
    }

    pub fn from_int(val: i32) -> Option<Self> {
        match val {
            0 => Some(Operation::PLUS),
            1 => Some(Operation::MINUS),
            2 => Some(Operation::MULTIPLY),
            3 => Some(Operation::DIVIDE),
            _ => None 
        }
    }
}

#[derive(Debug)]
pub struct Task {
    pub num_left: u64,
    pub num_right: u64,
    pub operation: Operation
}

impl Task {

    pub fn new(a: u64, b: u64, op: Operation) -> Self {
       Self {
           num_left: a,
           num_right: b,
           operation: op
       }
    }

    pub fn check_answer(&self, answer: f64) -> bool {
        let res = match self.operation {
            Operation::PLUS => (self.num_left + self.num_right) as f64,
            Operation::MINUS => (self.num_left - self.num_right) as f64,
            Operation::MULTIPLY => (self.num_left * self.num_right) as f64,
            Operation::DIVIDE => (self.num_left as f64) / (self.num_right as f64)
        };
        (res as f64 - answer).abs() < f64::EPSILON
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition() {
       let task = Task::new(10, 5, Operation::PLUS);
       assert!(task.check_answer(15.0));
       assert!(!task.check_answer(10.0));
    }

    #[test]
    fn test_subtraction() {
       let task = Task::new(10, 3, Operation::MINUS); 
       assert!(task.check_answer(7.0));
       assert!(!task.check_answer(13.0));
    }

    #[test]
    fn test_multiplication() {
       let task = Task::new(6, 7, Operation::MULTIPLY);
       assert!(task.check_answer(42.0));
       assert!(!task.check_answer(13.0));
    }

    #[test]
    fn test_division() {
       let task = Task::new(10, 2, Operation::DIVIDE);
       assert!(task.check_answer(5.0));
       assert!(!task.check_answer(2.0));
    }
}
