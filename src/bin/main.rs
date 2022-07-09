use std::time::Instant;

use puzzles::solver::hanoi::HanoiSolver;
use puzzles::solver::nquene::NqueneSolver;
use puzzles::solver::pentomino::PentominoSolver;
use puzzles::solver::sudoku::SudokuSolver;
use puzzles::solver::Solver;
fn main() {
    let run_sudoku = false;
    let run_hanoi = false;
    let run_queue = false;
    let run_pentomino = true;
    if run_sudoku {
        /////////////////////
        // 数独ソルバー
        /////////////////////
        let mut sudoku = SudokuSolver::new(vec![
            "000007000",
            "020008040",
            "103000000",
            "000150000",
            "000300070",
            "000000089",
            "090000000",
            "080002000",
            "000600100",
        ]);
        // let mut sudoku = SudokuSolver::new(vec![
        //     "902304501",
        //     "000208000",
        //     "758109423",
        //     "604005792",
        //     "000407000",
        //     "217900845",
        //     "106703904",
        //     "000501000",
        //     "509602317",
        // ]);
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
        let start = Instant::now();
        let _ = hanoi.all_run();
        let end = start.elapsed();
        println!(
            "simple: {}.{:03}s",
            end.as_secs(),
            end.subsec_nanos() / 1_000_000
        );
        hanoi.redo();
        let start = Instant::now();
        hanoi.all_par_run(4);
        let end = start.elapsed();
        println!(
            "par: {}.{:03}s",
            end.as_secs(),
            end.subsec_nanos() / 1_000_000
        );
    }
    if run_queue {
        println!("run_queue");
        let n = 11;
        let queue_solver = NqueneSolver::new(n);
        let start = Instant::now();
        queue_solver.simple();
        let end = start.elapsed();
        println!(
            "simple: {}.{:03}s",
            end.as_secs(),
            end.subsec_nanos() / 1_000_000
        );
        let start = Instant::now();
        queue_solver.par_simple();
        let end = start.elapsed();
        println!(
            "par: {}.{:03}s",
            end.as_secs(),
            end.subsec_nanos() / 1_000_000
        );
        let start = Instant::now();
        queue_solver.dfs();
        let end = start.elapsed();
        println!(
            "dfs: {}.{:03}s",
            end.as_secs(),
            end.subsec_nanos() / 1_000_000
        );
    }

    if run_pentomino {
        println!("pentomino");
        let mut solver = PentominoSolver::new(10, 6);
        solver.run().unwrap();
    }
    // println!("Hello, world!");
}
