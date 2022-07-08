use super::PuzzleResult;
use super::Solver;
#[derive(Debug)]
pub struct PentominoSolver {}

impl PentominoSolver {
    pub fn new() -> Self {
        PentominoSolver {}
    }
}

impl Solver for PentominoSolver {
    fn has_finished(&self) -> PuzzleResult<bool> {
        Ok(true)
    }
    fn search(&mut self) -> PuzzleResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::PentominoSolver;

    #[test]
    fn test() {
        let solver = PentominoSolver::new();
        println!("{:?}", solver);
    }
}
