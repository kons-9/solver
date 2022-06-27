mod solver;
use solver::hanoi::HanoiSolver;
use solver::sudoku::SudokuSolver;
use solver::Solver;
fn main() {
    let run_sudoku = false;
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
        let flag = sudoku.num_search();
        println!("{}", flag);
        let flag = sudoku.num_search();
        println!("{}", flag);
        let flag = sudoku.num_search();
        println!("{}", flag);
        let flag = sudoku.num_search();
        println!("{}", flag);
        let flag = sudoku.num_search();
        println!("{}", flag);
        let flag = sudoku.num_search();
        println!("{}", flag);
        println!("{}", sudoku);
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
    {
        /////////////////////
        // hanoi solver
        /////////////////////
        let n = 6;
        let mut hanoi = HanoiSolver::new(n);
        println!("count: {}", hanoi.count());
        let _ = hanoi.run().unwrap();
        println!("history: {:?}", hanoi.history);
        println!("towers: {:?}", hanoi.towers);
        println!("fin: {}", hanoi.has_finished().unwrap());

        hanoi.init();
        println!("count: {}", hanoi.count());
        let _ = hanoi.all_run();
        println!("history: {:?}", hanoi.history);
        println!("towers: {:?}", hanoi.towers);
        println!("fin: {}", hanoi.has_finished().unwrap());
        hanoi.redo();
        println!("fin: {}", hanoi.has_finished().unwrap());
    }

    // println!("Hello, world!");
}
