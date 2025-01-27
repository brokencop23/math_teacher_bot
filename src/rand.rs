use rand::Rng;


#[derive(Debug)]
pub enum RuleError {
    InvalidProbabilities,
    InvalidRange
}

#[derive(Debug)]
pub struct RandRule {
    from: u64,
    to: u64
}

impl RandRule {

    pub fn new(from: u64, to: u64) -> Result<Self, RuleError> {
        let rng = Self { from, to };
        rng.check()?;
        Ok(rng)
    }

    pub fn check(&self) -> Result<(), RuleError> { 
        if !self.valid_range() {
            return Err(RuleError::InvalidRange);
        }
        Ok(())
    }

    pub fn generate(&self) -> (u64, u64) {
        let mut rng = rand::thread_rng();
        let a: u64 = rng.gen_range(self.from..self.to);
        let b: u64 = rng.gen_range(self.from..self.to);
        (a, b)
    }

    fn valid_range(&self) -> bool {
        self.to > self.from
        && self.from > 0
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rng_num_rage() {
        let min = 1;
        let max = 5;
        let rule = RandRule::new(min, max).expect("err");
        let mut wrong_cases = 0;
        for _ in 1..100 {
            let (a, b) = rule.generate(); 
            if a > max || b > max || a < min || b < min {
                wrong_cases += 1;
            }
        }
        assert_eq!(wrong_cases, 0);
    }
}
