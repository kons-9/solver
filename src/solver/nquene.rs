use std::sync::{Arc, Mutex};

use super::{PuzzleError, PuzzleResult, Solver};
use rayon::prelude::*;

pub struct NqueneSolver {
    n: usize,
    anss: Vec<Vec<usize>>,
}
impl NqueneSolver {
    pub fn new(n: usize) -> Self {
        NqueneSolver {
            n,
            anss: Default::default(),
        }
    }
    pub fn simple(&self) -> Vec<Vec<usize>> {
        // n!通り調べる
        let mut nums: Vec<usize> = (0..self.n).into_iter().collect();
        let mut ans = Vec::new();
        Self::apply_permutation(&mut nums, self.n, Self::check_quene, &mut ans);
        ans
    }
    pub fn par_simple(&self) -> Arc<Mutex<Vec<Vec<usize>>>> {
        // n!通り調べる
        let mut nums: Vec<usize> = (0..self.n).into_iter().collect();
        let ans = Arc::new(Mutex::new(Vec::new()));
        Self::par_apply_permutation(&mut nums, self.n, Self::check_quene, ans.clone());
        ans
    }
    fn check_quene(quenes: &Vec<usize>) -> bool {
        for i in 0..quenes.len() {
            let qval = quenes[i];
            for j in (i + 1)..quenes.len() {
                let diff = j - i;
                if diff == qval.abs_diff(quenes[j]) {
                    return false;
                }
            }
        }
        true
    }
    fn par_apply_permutation(
        nums: &Vec<usize>,
        n: usize,
        f: fn(&Vec<usize>) -> bool,
        ans: Arc<Mutex<Vec<Vec<usize>>>>,
    ) {
        // apply_permutationを一段並行処理したバージョン
        // 順列を再帰的に生成してそれが条件fを満たすかどうか調べる
        // 満たした場合,ansに入れる
        // top + apply_permutation(n-1)
        if n == 1 {
            if f(nums) {
                ans.lock().unwrap().push(nums.clone())
            }
        }

        (0..n).into_par_iter().for_each(|i| {
            let mut nums = nums.clone();
            nums.swap(n - 1, i);

            // ここで並行処理ごとにtmpansを作らずアクセスすると
            // mutexのlockが何度も入ることになり遅くなる
            // Ng: (補助関数を作って)Self::_par_apply_permutation(&mut nums, n - 1, f, ans.clone);
            let mut tmpans = Vec::new();
            Self::apply_permutation(&mut nums, n - 1, f, &mut tmpans);
            ans.lock().unwrap().extend(tmpans);
        });
    }
    fn apply_permutation(
        nums: &mut Vec<usize>,
        n: usize,
        f: fn(&Vec<usize>) -> bool,
        ans: &mut Vec<Vec<usize>>,
    ) {
        // 順列を再帰的に生成してそれが条件fを満たすかどうか調べる
        // 満たした場合,ansに入れる
        // top + apply_permutation(n-1)
        if n == 1 {
            // println!("{:?}", nums);
            if f(nums) {
                ans.push(nums.clone())
            }
        }

        for i in 0..n {
            nums.swap(n - 1, i);
            Self::apply_permutation(nums, n - 1, f, ans);
            nums.swap(n - 1, i);
        }
    }
}
impl Solver for NqueneSolver {
    fn has_finished(&self) -> PuzzleResult<bool> {
        Ok(self.anss.len() != 0)
    }
    fn search(&mut self) -> Result<(), PuzzleError> {
        self.anss = self.simple();
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::NqueneSolver;

    #[test]
    fn test_par_quene() {
        let anslen = [1, 0, 0, 2, 10, 4, 40, 92, 352, 724];
        // let n = 8;
        for n in 1..=anslen.len() {
            let solver = NqueneSolver::new(n);
            let ans = solver.par_simple();
            // println!("{:?}", ans);
            assert!(ans.lock().unwrap().len() == anslen[n - 1]);
        }
    }
    #[test]
    fn test_quene() {
        let anslen = [1, 0, 0, 2, 10, 4, 40, 92, 352, 724];
        // let n = 8;
        for n in 1..=anslen.len() {
            let solver = NqueneSolver::new(n);
            let ans = solver.simple();
            // println!("{:?}", ans);
            assert!(ans.len() == anslen[n - 1]);
        }
    }
}
