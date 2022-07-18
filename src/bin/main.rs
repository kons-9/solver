use std::fmt::Display;
use std::time::{Duration, Instant};

use puzzles::solver::hanoi::HanoiSolver;
use puzzles::solver::nquene::NqueneSolver;
use puzzles::solver::pentomino::PentominoSolver;
use puzzles::solver::sudoku::SudokuSolver;
use puzzles::solver::Solver;

struct Time {
    instant: Instant,
    duration: Duration,
}
impl Time {
    fn new() -> Self {
        let instant = Instant::now();
        let duration = instant.elapsed();
        Time { instant, duration }
    }
    fn start(&mut self) {
        self.instant = Instant::now();
    }
    fn end(&mut self) {
        self.duration = self.instant.elapsed();
    }
}
impl Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{:03}s",
            self.duration.as_secs(),
            self.duration.subsec_nanos()
        )
    }
}
fn main() {
    // 実行サンプル
    // それぞれのflagの場所に実装がある
    let run_sudoku = false;
    let run_hanoi = false;
    let run_queue = false;
    let run_pentomino = true;

    let mut timer = Time::new();

    if run_sudoku {
        /////////////////////
        // 数独ソルバー
        /////////////////////
        let mut sudoku = SudokuSolver::new(vec![
            "902304501",
            "000208000",
            "758109423",
            "604005792",
            "000407000",
            "217900845",
            "106703904",
            "000501000",
            "509602317",
        ]);
        println!("{}", sudoku);
        let flag = sudoku.num_search();
        println!("{}", flag);
        let flag = sudoku.num_search();
        println!("{}", flag);
        sudoku.search().unwrap();
        println!("{}", sudoku);
        println!("{}", sudoku.has_finished().unwrap());
        let ans = SudokuSolver::new(vec![
            "962374581",
            "341258679",
            "758169423",
            "634815792",
            "895427136",
            "217936845",
            "126783954",
            "473591268",
            "589642317",
        ]);
        println!("{}", sudoku == ans);
    }
    if run_hanoi {
        /////////////////////
        // hanoi solver
        /////////////////////
        let n = 23;

        let mut hanoi = HanoiSolver::new(n);
        // println!("count: {}", hanoi.count());
        // let _ = hanoi.run().unwrap();
        // println!("history: {:?}", hanoi.history);
        // println!("towers: {:?}", hanoi.towers);
        // println!("fin: {}", hanoi.has_finished().unwrap());
        // hanoi.redo();
        // println!("fin: {}", hanoi.has_finished().unwrap());

        hanoi.init();

        timer.start();
        let _ = hanoi.all_run();
        timer.end();

        println!("simple: {}", timer);

        hanoi.redo();

        timer.start();
        hanoi.all_par_run(4);
        timer.end();

        println!("par: {}", timer);
    }
    if run_queue {
        println!("run_queue");
        let n = 11;
        let queue_solver = NqueneSolver::new(n);

        timer.start();
        queue_solver.simple();
        timer.end();
        println!("simple: {}", timer);

        timer.start();
        queue_solver.par_simple();
        timer.end();
        println!("par: {}", timer);

        timer.start();
        queue_solver.dfs();
        timer.end();
        println!("dfs: {}", timer);
    }

    if run_pentomino {
        println!("pentomino");
        let mut solver = PentominoSolver::new(10, 6);
        println!("{}", solver.run_all());
        solver.run().unwrap();
        println!("{}", solver);
        solver.init();
        println!("{}", solver.run_all());

        let solver = PentominoSolver::meiji_black(puzzles::solver::pentomino::TargetType::ROTATE);
        println!("{}", solver.search_one_ans());
        println!("{}", solver);
        solver.init();
        println!("{}", solver.run_all());
    }
}
