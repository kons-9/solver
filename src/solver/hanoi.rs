use std::{cmp::min, collections::BTreeSet};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use super::{PuzzleError, PuzzleResult, Solver};
type Data = usize;
#[derive(Debug)]
pub struct HanoiSolver {
    pub towers: [BTreeSet<Data>; 3],
    /// towerの大きさ
    n: usize,
    /// 現在何回目の操作が終わったか
    state: u32,
    /// 履歴(from, to, val)
    pub history: Vec<(usize, usize, Data)>,
    /// countのcache，
    cache_count: Vec<u32>,
}

impl HanoiSolver {
    pub fn new(n: usize) -> Self {
        let mut heap = BTreeSet::new();
        let mut cache_count = vec![0];
        for i in 1..=n {
            heap.insert(i);
            cache_count.push(Self::_opt_count(i));
        }
        HanoiSolver {
            towers: [heap, BTreeSet::new(), BTreeSet::new()],
            n,
            state: 0,
            history: Vec::new(),
            cache_count,
        }
    }
    /// 初期化する
    pub fn init(&mut self) {
        let mut heap = BTreeSet::new();
        for i in 1..=self.n {
            heap.insert(i);
        }
        self.history = Vec::new();
        self.towers = [heap, BTreeSet::new(), BTreeSet::new()];
        self.state = 0;
    }
    /// 一手巻き戻す
    pub fn redo(&mut self) {
        if self.state == 0 {
            eprintln!("The state have initiarized");
        }
        if let Some((from, to, val)) = self.history.pop() {
            // 逆に適用する
            self.move_val(to, from, val).unwrap();
            self.state -= 1;
        }
    }
    /// 最適な行動をn回した後の状態を表す
    pub fn opt_behaiver(&mut self, cnt: u32) -> PuzzleResult<()> {
        let state = self.state;
        for _ in state..min(cnt + state, self.count()) {
            let (from, to, val) = self.next_from_to();
            self.move_val(from, to, val)?;
            self.state += 1;
        }
        Ok(())
    }
    /// tower[from]からtower[to]へvalを移動させる。
    /// 移動できるのはそのタワーでの最小値のみで次のタワーでも最小値になるもののみ
    pub fn move_val(&mut self, from: usize, to: usize, val: Data) -> PuzzleResult<Data> {
        // ガード

        if from > 2 && to > 2 {
            return Err(PuzzleError::new(format!(
                "invalid index: from:{}, to:{}: they must be in [0,2]",
                from, to
            )));
        }
        if self.towers[from].len() == 0 {
            return Err(PuzzleError::new(format!(
                "tower of from is empty.: from: {}, to: {}, val: {}, {:?}",
                from, to, val, self
            )));
        }
        if self.towers[to].len() > 0
            && self.towers[from].iter().next() > self.towers[to].iter().next()
        {
            return Err(PuzzleError::new(format!(
                "cannot stack the tower: from: {}, to: {}, val: {}, {:?}",
                from, to, val, self
            )));
        }
        if Some(&val) != self.towers[from].iter().next() {
            // fromから撮ってきた値と等しくなければおかしい
            return Err(PuzzleError::new(format!(
                "invalid value: val must equal to towers[from]'s top, but val: {}, top: {}",
                val,
                self.towers[from].iter().next().unwrap()
            )));
        }

        // fromから値を削除
        self.towers[from].remove(&val);
        self.towers[to].insert(val);
        self.history.push((from, to, val));
        Ok(val)
    }

    /// 一度に全ての実行を行う
    pub fn all_run(&mut self) {
        self.history = Self::_all_run(0, 2, self.n);
        let mut tmp = BTreeSet::new();
        std::mem::swap(&mut self.towers[2], &mut tmp);
        std::mem::swap(&mut self.towers[0], &mut tmp);
        std::mem::swap(&mut self.towers[2], &mut tmp);
        self.state = self.count();
    }
    /// all_runの補助関数。再起関数で実装
    fn _all_run(from: usize, to: usize, val: Data) -> Vec<(usize, usize, Data)> {
        if val == 1 {
            return vec![(from, to, val)];
        }
        let mut vec = Vec::new();
        vec.extend(Self::_all_run(from, 3 - from - to, val - 1));
        vec.push((from, to, val));
        vec.extend(Self::_all_run(3 - from - to, to, val - 1));
        vec
    }

    pub fn all_par_run(&mut self, par_num: u32) {
        self.history = Self::_all_par_run(0, 2, self.n, par_num);
        let mut tmp = BTreeSet::new();
        std::mem::swap(&mut self.towers[2], &mut tmp);
        std::mem::swap(&mut self.towers[0], &mut tmp);
        std::mem::swap(&mut self.towers[2], &mut tmp);
        self.state = self.count();
    }

    /// 並行処理を行う
    fn _all_par_run(
        from: usize,
        to: usize,
        val: Data,
        mut par_num: u32,
    ) -> Vec<(usize, usize, Data)> {
        assert_ne!(par_num, 0);
        // par_numは2の累乗になるように設定
        let par_rec_num = {
            let mut cnt = 0;
            while par_num != 0 {
                cnt += 1;
                par_num /= 2
            }
            cnt
        };
        fn make_vals(from: usize, to: usize, val: usize) -> Vec<(usize, usize, Data, bool)> {
            if val == 1 {
                vec![(from, to, val, false)]
            } else {
                vec![
                    (from, 3 - from - to, val - 1, true),
                    (from, to, val, false),
                    (3 - from - to, to, val - 1, true),
                ]
            }
        }

        // (usize,usize,Data,bool): from, to, data, flag
        let mut all_run_args = vec![(from, to, val, true)];

        for _ in 0..par_rec_num {
            let mut new_all_run_args = Vec::new();
            for x @ (from, to, val, flag) in all_run_args {
                if flag {
                    new_all_run_args.extend(make_vals(from, to, val));
                } else {
                    new_all_run_args.push(x);
                }
            }
            all_run_args = new_all_run_args;
        }
        let mut ans = Vec::new();
        let anss: Vec<Vec<(usize, usize, Data)>> = all_run_args
            .par_iter()
            .map(|&(from, to, data, flag)| {
                if flag {
                    Self::_all_run(from, to, data)
                } else {
                    vec![(from, to, data)]
                }
            })
            .collect();
        for i in anss {
            ans.extend(i);
        }
        ans
    }

    // 次の値を探す
    fn next_from_to(&mut self) -> (usize, usize, Data) {
        self._find_next_from_to_val((0, 2, self.n), self.state)
    }
    // 次の値を探す補助関数
    // 最適な行動は中心から対照的な木構造になるため再起的に探索
    fn _find_next_from_to_val(
        &self,
        (from, to, n): (usize, usize, Data),
        pos: u32,
    ) -> (usize, usize, Data) {
        // f(n)の中で何番目のposか
        // f(n) = f(n-1) ++ n ++ f(n-1)という形で操作する順番は決まっている。
        // たかだかself.n回
        if n == 1 {
            return (from, to, 1);
        }

        let count = self.cache_count[n];
        let center = count / 2;

        if pos == center {
            (from, to, n)
        } else if pos < center {
            // 小さい時はfromが一緒で行き先がtoでもfromでもないindexを指定する
            self._find_next_from_to_val((from, 3 - from - to, n - 1), pos)
        } else {
            // 大きい時はtoが一緒で元の場所がtoでもfromでもないindexを指定する
            self._find_next_from_to_val((3 - from - to, to, n - 1), pos - 1 - center)
        }
    }
    ////////////////////
    // count functions
    ////////////////////
    pub fn count(&self) -> u32 {
        self.cache_count[self.n]
    }
    /// 単純な再帰実装
    fn _rec_count(n: usize) -> u32 {
        // 再起関数バージョン
        match n {
            0 => 0,
            // 真ん中にn-1動かし，一番右に一番大きいものを動かし，またn-1個動かせば良い
            n => 2 * Self::_rec_count(n - 1) + 1,
        }
    }
    /// 末尾最適化を明示的にした
    fn _tail_rec_count(n: usize) -> u32 {
        // 末尾最適を明示的に書いたcount
        Self::__tail_rec_count(n, 0)
    }
    /// 補助関数
    fn __tail_rec_count(n: usize, val: u32) -> u32 {
        match n {
            0 => val,
            n => Self::__tail_rec_count(n - 1, 2 * val + 1),
        }
    }
    /// forloopでの実装
    fn _for_count(n: usize) -> u32 {
        // forバージョン
        let mut count = 0;
        for _ in 1..=n {
            count = 2 * count + 1
        }
        count
    }

    /// 簡単な漸化式を解くと一発でも止まるあため直接求める実装
    fn _opt_count(n: usize) -> u32 {
        2_u32.pow(n as u32) - 1
    }
}

impl Solver for HanoiSolver {
    fn has_finished(&self) -> PuzzleResult<bool> {
        Ok(self.towers[2].len() == self.n)
    }
    fn search(&mut self) -> Result<(), PuzzleError> {
        let (from, to, val) = self.next_from_to();
        let _ = self.move_val(from, to, val)?;
        self.state += 1;
        // println!("from: {}, to: {}, val: {}", from, to, x);
        Ok(())
    }
    fn run(&mut self) -> Result<(), PuzzleError> {
        self.opt_behaiver(self.count())
    }
}

#[cfg(test)]
mod test {
    use crate::solver::Solver;

    use super::HanoiSolver;

    #[test]
    fn test_next_val() {
        let n = 10;
        let solver = HanoiSolver::new(n);
        let ans = [
            1, 2, 1, 3, 1, 2, 1, 4, 1, 2, 1, 3, 1, 2, 1, 5, 1, 2, 1, 3, 1, 2, 1, 4, 1, 2, 1, 3, 1,
            2, 1,
        ];
        for i in 1..ans.len() {
            let (_, _, val) = solver._find_next_from_to_val((0, 2, n), i as u32);
            assert!(ans[i] == val, "i: {}, ans: {}, val: {}", i, ans[i], val);
        }
    }
    #[test]
    fn test_next_from_to() {
        let mut solver = HanoiSolver::new(2);
        let ans = [(0, 1, 1), (0, 2, 2), (1, 2, 1)];
        for a in ans {
            println!("{:?}", solver.next_from_to());
            let (from, to, val) = solver.next_from_to();
            assert!(
                (from, to, val) == a,
                "from: {}, to: {}, val: {}, a: {:?} {:?}",
                from,
                to,
                val,
                a,
                solver
            );
            solver.state += 1;
        }
        println!();
        let mut solver = HanoiSolver::new(3);
        let ans = [
            (0, 2, 1),
            (0, 1, 2),
            (2, 1, 1),
            (0, 2, 3),
            (1, 0, 1),
            (1, 2, 2),
            (0, 2, 1),
        ];
        for a in ans {
            println!("{:?}", solver.next_from_to());
            let (from, to, val) = solver.next_from_to();
            solver.move_val(from, to, val).unwrap();
            assert!(
                (from, to, val) == a,
                "from: {}, to: {}, val: {}, a: {:?} {:?}",
                from,
                to,
                val,
                a,
                solver
            );
            solver.state += 1;
        }
    }
    #[test]
    fn test_count() {
        let solver = HanoiSolver::new(10);
        for i in 0..solver.cache_count.len() {
            assert!(HanoiSolver::_opt_count(i) == solver.cache_count[i]);
            assert!(HanoiSolver::_opt_count(i) == HanoiSolver::_rec_count(i));
            assert!(HanoiSolver::_opt_count(i) == HanoiSolver::_for_count(i));
            assert!(HanoiSolver::_opt_count(i) == HanoiSolver::_tail_rec_count(i));
        }
        assert!(HanoiSolver::_opt_count(solver.n) == solver.count());
    }
    #[test]
    fn test_par_run_all() {
        let n = 10;
        let mut solver = HanoiSolver::new(n);
        solver.all_run();
        let mut solver2 = HanoiSolver::new(n);
        solver2.all_par_run(4);
        assert_eq!(solver.history, solver2.history);
    }
    #[test]
    fn test_run_all() {
        let n = 10;
        let mut solver = HanoiSolver::new(n);
        solver.all_run();
        let mut solver2 = HanoiSolver::new(n);
        solver2.run().unwrap();
        assert!(solver.history == solver2.history);
    }
}
