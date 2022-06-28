use super::PuzzleResult;
use super::Solver;
#[derive(Debug)]
pub struct FifteenPuzzleSolver {}

impl FifteenPuzzleSolver {
    pub fn new() -> Self {
        FifteenPuzzleSolver {}
    }
}

impl Solver for FifteenPuzzleSolver {
    fn has_finished(&self) -> PuzzleResult<bool> {
        Ok(true)
    }
    fn search(&mut self) -> PuzzleResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::FifteenPuzzleSolver;

    #[test]
    fn test() {
        let solver = FifteenPuzzleSolver::new();
        println!("{:?}", solver);
    }
}
