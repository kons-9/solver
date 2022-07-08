use std::fmt::Display;

pub mod fifteen_puzzle;
pub mod hanoi;
pub mod nquene;
pub mod pentomino;
pub mod sudoku;

#[derive(Debug)]
pub struct PuzzleError {
    error: String,
}
impl PuzzleError {
    fn new(str: impl Into<String>) -> Self {
        let error = str.into();
        PuzzleError { error }
    }
}
impl Display for PuzzleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}
type PuzzleResult<T> = Result<T, PuzzleError>;

pub trait Solver {
    /// パズルを実行する関数
    fn run(&mut self) -> Result<(), PuzzleError> {
        while !self.has_finished()? {
            self.search()?;
        }
        Ok(())
    }

    /// 解となり得る選択肢が一つも見つからなかったらPuzzleErrorを返す。
    /// そうでない時は適当な探索をステップ実行する
    fn search(&mut self) -> PuzzleResult<()>;

    /// 終了状態であればtrueを返す
    fn has_finished(&self) -> PuzzleResult<bool>;
}
