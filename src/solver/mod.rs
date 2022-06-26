use std::fmt::Display;

pub mod sudoku;

#[derive(Debug)]
pub struct PazzleError {
    error: String,
}
impl PazzleError {
    fn new(str: impl Into<String>) -> Self {
        let error = str.into();
        PazzleError { error }
    }
}
impl Display for PazzleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}
type PazzleResult<T> = Result<T, PazzleError>;

pub trait Solver {
    /// パズルを実行する関数
    fn run(&mut self) -> Result<(), PazzleError> {
        while !self.has_finished()? {
            self.search()?;
        }
        Ok(())
    }

    /// 解となり得る選択肢が一つも見つからなかったらPazzleErrorを返す。
    /// そうでない時は適当な探索をステップ実行する
    fn search(&mut self) -> Result<(), PazzleError>;

    /// 終了状態であればtrueを返す
    fn has_finished(&self) -> PazzleResult<bool>;
}
