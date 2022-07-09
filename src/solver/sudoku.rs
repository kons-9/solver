use super::PuzzleError;
use super::PuzzleResult;
use super::Solver;
use core::panic;
use rayon::prelude::*;
use std::collections::BTreeSet;
use std::fmt::Display;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;

type Data = usize;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum FieldElement {
    SUG(BTreeSet<Data>),
    /// 確定したマス
    NUM(Data),
}

impl FieldElement {
    /// suggestでnumに変更する場合のみ変更した値を，それ以外は0を返す
    pub fn suggest_confirmed(&mut self) -> Data {
        let val = match self {
            FieldElement::SUG(x) => {
                if x.len() == 1 {
                    let val = *x.iter().next().unwrap();
                    assert!(val > 0);
                    *self = FieldElement::NUM(val);
                    val
                } else {
                    0
                }
            }
            _ => 0,
        };
        val
    }
}

#[derive(Clone, Debug)]
pub struct SudokuSolver {
    pub field: Vec<Vec<Arc<Mutex<FieldElement>>>>,
    row_cache: [Arc<Mutex<[bool; 9]>>; 9],
    col_cache: [Arc<Mutex<[bool; 9]>>; 9],
    block_cache: [Arc<Mutex<[bool; 9]>>; 9],
}
impl PartialEq for SudokuSolver {
    fn eq(&self, other: &Self) -> bool {
        self.field
            .iter()
            .zip(&other.field)
            .map(|(s, o)| {
                for (s, o) in s.iter().zip(o) {
                    if *s.lock().unwrap() != *o.lock().unwrap() {
                        return false;
                    }
                }
                true
            })
            .fold(true, |x, y| x & y)
    }
}
impl Eq for SudokuSolver {}

// related to block
#[inline]
fn xy_to_block_ind(x: usize, y: usize) -> usize {
    (x / 3) * 3 + y / 3
}
static BLOCK_LEFT_UPPER_CORNER: [(usize, usize); 9] = [
    (0, 0),
    (0, 3),
    (0, 6),
    (3, 0),
    (3, 3),
    (3, 6),
    (6, 0),
    (6, 3),
    (6, 6),
];
static BLOCK_MOV: [(usize, usize); 9] = [
    (0, 0),
    (0, 1),
    (0, 2),
    (1, 0),
    (1, 1),
    (1, 2),
    (2, 0),
    (2, 1),
    (2, 2),
];

impl SudokuSolver {
    pub fn new(string: Vec<&str>) -> Self {
        let mut field = Vec::new();
        let default_tree = FieldElement::SUG(BTreeSet::from([1, 2, 3, 4, 5, 6, 7, 8, 9]));
        let row_cache: [Arc<Mutex<[bool; 9]>>; 9] = Default::default();
        let col_cache: [Arc<Mutex<[bool; 9]>>; 9] = Default::default();
        let block_cache: [Arc<Mutex<[bool; 9]>>; 9] = Default::default();

        for (xind, i) in string.iter().enumerate() {
            field.push(Vec::new());
            if i.len() != 9 {
                panic!("invalid data");
            }
            for c in i.chars() {
                match c {
                    '*' | '0' => field[xind].push(Arc::new(Mutex::new(default_tree.clone()))),
                    n @ '1'..='9' => {
                        let val = n.to_digit(10).unwrap() as Data;
                        field[xind].push(Arc::new(Mutex::new(FieldElement::NUM(val))));
                        // (*row_cache[xind].lock().unwrap())[val - 1] = true;
                        // (*col_cache[yind].lock().unwrap())[val - 1] = true;
                        // (*block_cache[xy_to_block_ind(xind, yind)].lock().unwrap())[val - 1] = true
                    }
                    _ => panic!("invalid data"),
                }
            }
        }

        let mut solver = SudokuSolver {
            field,
            row_cache,
            col_cache,
            block_cache,
        };
        solver.init();
        solver
    }
    fn init(&mut self) {}

    /// cacheの変更があるかどうかを返す
    fn col_search(&self, col: usize, num: Data) -> bool {
        let mut confirm_row = None;
        {
            // cacheが有効な期間
            let cache = &mut self.col_cache[col].lock().unwrap()[num - 1];
            if *cache {
                // 前までの探索で確定済み
                return false;
            }
            for row in 0..9 {
                let elem = self.field[row][col].lock().unwrap();

                match &*elem {
                    FieldElement::NUM(n) if n == &num => {
                        confirm_row = Some(row);
                        *cache = true;
                        break;
                    }
                    FieldElement::SUG(set) if set.len() == 1 && set.contains(&num) => {
                        confirm_row = Some(row);
                        *cache = true;
                        break;
                    }
                    _ => (),
                };
            }
            if !*cache {
                return false;
            }
        }
        // ここまで来れるのはflagがTrueかつcacheがFalseだった場合のみ
        let confirm_row = confirm_row.unwrap();
        for row in 0..9 {
            let mut elem = self.field[row][col].lock().unwrap();
            match &mut *elem {
                FieldElement::SUG(set) => {
                    if confirm_row != row {
                        set.remove(&num);
                    }
                }
                _ => (),
            }
        }
        true
    }
    /// cacheの変更があるかどうかを返す
    fn block_search(&self, block: usize, num: Data) -> bool {
        let mut confirm_col_row = None;
        {
            // cacheが有効な期間
            let cache = &mut self.block_cache[block].lock().unwrap()[num - 1];
            if *cache {
                // 前までの探索で確定済み
                return false;
            }
            let (lrow, lcol) = BLOCK_LEFT_UPPER_CORNER[block];
            for (drow, dcol) in BLOCK_MOV {
                let row = lrow + drow;
                let col = lcol + dcol;
                let elem = self.field[row][col].lock().unwrap();

                match &*elem {
                    FieldElement::NUM(n) if n == &num => {
                        confirm_col_row = Some((col, row));
                        *cache = true;
                        break;
                    }
                    FieldElement::SUG(set) if set.len() == 1 && set.contains(&num) => {
                        confirm_col_row = Some((col, row));
                        *cache = true;
                        break;
                    }
                    _ => (),
                };
            }
            if !*cache {
                return false;
            }
        }
        // ここまで来れるのはflagがTrueかつcacheがFalseだった場合のみ
        let confirm_col_row = confirm_col_row.unwrap();
        let (lrow, lcol) = BLOCK_LEFT_UPPER_CORNER[block];
        for (drow, dcol) in BLOCK_MOV {
            let row = lrow + drow;
            let col = lcol + dcol;
            let mut elem = self.field[row][col].lock().unwrap();
            match &mut *elem {
                FieldElement::SUG(set) => {
                    if confirm_col_row != (col, row) {
                        set.remove(&num);
                    }
                }
                _ => (),
            }
        }
        true
    }
    /// cacheの変更があるかどうかを返す
    fn row_search(&self, row: usize, num: Data) -> bool {
        let mut confirm_col = None;
        {
            // cacheが有効な期間
            let cache = &mut self.row_cache[row].lock().unwrap()[num - 1];
            if *cache {
                // 前までの探索で確定済み
                // panic!("{:?}", cache);
                return false;
            }
            for col in 0..9 {
                let elem = self.field[row][col].lock().unwrap();

                match &*elem {
                    FieldElement::NUM(n) if n == &num => {
                        *cache = true;
                        confirm_col = Some(col);
                        break;
                    }
                    FieldElement::SUG(set) if set.len() == 1 && set.contains(&num) => {
                        *cache = true;
                        confirm_col = Some(col);
                        break;
                    }
                    _ => (),
                };
            }
            if !*cache {
                return false;
            }
        }
        // ここまで来れるのはflagがTrueかつcacheがFalseだった場合のみ
        let confirm_col = confirm_col.unwrap();
        for col in 0..9 {
            let mut elem = self.field[row][col].lock().unwrap();
            match &mut *elem {
                FieldElement::SUG(set) => {
                    if confirm_col != col {
                        set.remove(&num);
                    }
                }
                _ => (),
            }
        }
        true
    }

    #[allow(dead_code)]
    /// par version of one_line_search
    /// but I think xxx_search is not heavy task, so I think it don't make the runtime short.
    fn par_one_line_search(&self, num: Data) -> bool {
        let mut flag = false;
        // row
        flag |= (0..9)
            .into_par_iter()
            .map(|row| self.row_search(row, num))
            .reduce(|| false, |a, b| a | b);
        // col
        flag |= (0..9)
            .into_par_iter()
            .map(|col| self.col_search(col, num))
            .reduce(|| false, |a, b| a | b);
        //block
        flag |= (0..9)
            .into_par_iter()
            .map(|block| self.block_search(block, num))
            .reduce(|| false, |a, b| a | b);
        flag
    }
    /// normal version of one_line_search
    fn one_line_search(&self, num: Data) -> bool {
        let mut flag = false;
        // row
        flag |= (0..9)
            .into_iter()
            .map(|row| self.row_search(row, num))
            .fold(false, |a, b| a | b);
        // col
        flag |= (0..9)
            .into_iter()
            .map(|col| self.col_search(col, num))
            .fold(false, |a, b| a | b);
        //block
        flag |= (0..9)
            .into_iter()
            .map(|block| self.block_search(block, num))
            .fold(false, |a, b| a | b);
        flag
    }
    fn line_confirmed(&self, num: Data) -> bool {
        // 変化するものが一つでもあるかどうか
        let mut flag = false;
        // blockごとに回す
        for (lx, ly) in BLOCK_LEFT_UPPER_CORNER {
            // falseで埋める
            let mut rowtrees: [bool; 3] = Default::default();
            let mut coltrees: [bool; 3] = Default::default();

            for dy in 0..3 {
                for dx in 0..3 {
                    let y = ly + dy;
                    let x = lx + dx;
                    let mut elem = self.field[x][y].lock().unwrap();
                    match &mut *elem {
                        FieldElement::SUG(set) => {
                            let is_contain = set.contains(&num);
                            rowtrees[dx] |= is_contain;
                            coltrees[dy] |= is_contain;
                        }
                        _ => {}
                    }
                }
            }
            for (target, sub, sub2) in [(0, 1, 2), (1, 2, 0), (2, 0, 1)] {
                let row_target_tree = rowtrees[target];
                let col_target_tree = coltrees[target];
                let row_other_tree = rowtrees[sub] | rowtrees[sub2];
                let col_other_tree = coltrees[sub] | coltrees[sub2];
                // targetにあってotherにないものを探す
                let row_flag = row_target_tree & !row_other_tree;
                let col_flag = col_target_tree & !col_other_tree;

                if row_flag {
                    let row = lx + target;
                    for col in 0..9 {
                        if (col / 3) * 3 == ly {
                            // 同一ブロックなので消さない
                            continue;
                        }
                        let mut elem = self.field[row][col].lock().unwrap();
                        flag |= match &mut *elem {
                            FieldElement::SUG(set) => set.remove(&num),
                            _ => false,
                        }
                    }
                }
                if col_flag {
                    let col = ly + target;
                    for row in 0..9 {
                        if (row / 3) * 3 == lx {
                            // 同一ブロックなので消さない
                            continue;
                        }
                        let mut elem = self.field[row][col].lock().unwrap();
                        flag |= match &mut *elem {
                            FieldElement::SUG(set) => set.remove(&num),
                            _ => false,
                        }
                    }
                }
            }
        }
        flag
    }
    #[allow(dead_code)]
    fn all_line_confirmed(&self) -> bool {
        let mut flag = false;
        for (lx, ly) in BLOCK_LEFT_UPPER_CORNER {
            let mut rowtrees: [BTreeSet<usize>; 3] = Default::default();
            let mut coltrees: [BTreeSet<usize>; 3] = Default::default();
            for dy in 0..3 {
                for dx in 0..3 {
                    let y = ly + dy;
                    let x = lx + dx;
                    let mut elem = self.field[x][y].lock().unwrap();
                    match &mut *elem {
                        FieldElement::SUG(set) => {
                            rowtrees[dx].append(set);
                            coltrees[dy].append(set);
                        }
                        _ => {}
                    }
                }
            }
            for (target, sub, sub2) in [(0, 1, 2), (1, 2, 0), (2, 0, 1)] {
                let row_target_tree = &rowtrees[target];
                let col_target_tree = &coltrees[target];
                let row_other_tree1 = &rowtrees[sub];
                let col_other_tree1 = &coltrees[sub];
                let row_other_tree2 = &rowtrees[sub2];
                let col_other_tree2 = &coltrees[sub2];
                // targetにあってotherにないものを探す
                let row_vals =
                    row_target_tree & &(row_target_tree ^ &(row_other_tree1 | row_other_tree2));
                let col_vals =
                    col_target_tree & &(col_target_tree ^ &(col_other_tree1 | col_other_tree2));

                for val in row_vals {
                    let row = lx + target;
                    for col in 0..9 {
                        if (lx, ly) == BLOCK_LEFT_UPPER_CORNER[xy_to_block_ind(row, col)] {
                            continue;
                        }
                        let mut elem = self.field[row][col].lock().unwrap();
                        flag |= match &mut *elem {
                            FieldElement::SUG(set) => set.remove(&val),
                            _ => false,
                        }
                    }
                }
                for val in col_vals {
                    let col = ly + target;
                    for row in 0..9 {
                        if (lx, ly) == BLOCK_LEFT_UPPER_CORNER[xy_to_block_ind(row, col)] {
                            continue;
                        }
                        let mut elem = self.field[row][col].lock().unwrap();
                        flag |= match &mut *elem {
                            FieldElement::SUG(set) => set.remove(&val),
                            _ => false,
                        }
                    }
                }
            }
        }
        flag
    }
    fn pseudo_confirmed(&self, num: usize) -> bool {
        let mut flag = false;
        for block in 0..BLOCK_LEFT_UPPER_CORNER.len() {
            flag |= self.block_pseudo_confirmed(num, block);
        }
        flag
    }
    fn block_pseudo_confirmed(&self, num: usize, block: usize) -> bool {
        // todo(ちょいむずい...?)
        let (lx, ly) = BLOCK_LEFT_UPPER_CORNER[block];

        // blockのx,y座標と，そのマスに含まれる数字を記録する
        let mut set_in_num_xy = Vec::new();

        for (dx, dy) in BLOCK_MOV {
            let x = lx + dx;
            let y = ly + dy;
            let elem = self.field[x][y].lock().unwrap();
            match &*elem {
                FieldElement::NUM(n) if n == &num => {
                    // numが確定済みの値なのでスルー
                    return false;
                }
                FieldElement::SUG(set) if set.contains(&num) => {
                    // blockのx,y座標と，そのマスに含まれる数字を記録する
                    set_in_num_xy.push((x, y));
                }
                _ => (),
            }
        }

        // 計算量削減のためset_in_num_xyの上限を設けておく。
        // 多分人の手でも3くらいがいいとこ？
        const LIMIT: usize = 4;
        // bit演算のための補助関数
        // https://qiita.com/pikohideaki/items/cf0f1d4dd7fb57c7aa5b
        fn next_combination(bit: i32) -> usize {
            let x = bit & -bit;
            let y = x + bit;
            return (y | (((bit & !y) / x) >> 1)) as usize;
        }
        // 変更があるかどうかのフラグ
        let mut flag = false;

        // n個のマスにn個の種類の数字のみしか入らないので他のマスにある数字を削除
        // kこの部分集合の中にk個あれば他のマスにこれらの数字はは入れない
        // kはlimitを上限にする
        for (k, vec) in (2..LIMIT).map(|k| {
            let mut ret = Vec::new();
            let mut bit = (1 << k) - 1;
            while bit < (1 << set_in_num_xy.len()) {
                ret.push(bit);
                bit = next_combination(bit as i32);
            }
            (k, ret)
        }) {
            // kこの部分集合
            'BIT: for bit in vec {
                // println!("k:{}, {:b}, {:?}", k, bit, set_in_num_xy);
                let mut num_set = BTreeSet::new();
                // kこの部分集合に含まれるものの数を数える
                for i in 0..set_in_num_xy.len() {
                    // set_in_num_xyからkこのマスを選択する
                    if (bit >> i) & 1 == 1 {
                        let (x, y) = set_in_num_xy[i];
                        let elem = self.field[x][y].lock().unwrap();
                        match &*elem {
                            FieldElement::SUG(set) => {
                                num_set = &num_set | set;
                            }
                            FieldElement::NUM(_) => {
                                panic!("unreachable");
                            }
                        }
                    }
                }
                if num_set.len() != k {
                    continue 'BIT;
                }
                // ここまできたということはcontinue 'BITにならなかった
                // 現在の部分集合が条件を満たしている
                for i in 0..set_in_num_xy.len() {
                    // set_in_num_xyからkこ以外のマスを選択する
                    if (bit >> i) & 1 == 0 {
                        let (x, y) = set_in_num_xy[i];
                        let mut elem = self.field[x][y].lock().unwrap();
                        match &mut *elem {
                            FieldElement::SUG(set) => {
                                let dif = &*set - &num_set;
                                // println!("dif: {:?},set: {:?}", dif, set);
                                if dif.len() != set.len() {
                                    // 変更があったのでflagをtrueにする
                                    flag = true;
                                    *set = dif;
                                }
                            }
                            _ => (),
                        }
                    }
                }
            }
        }
        // n個のマスにのみn種類の数字があるためその他の種類の数字を削除(todo)
        flag
    }

    /// 私が数独を解くのに使うアルゴリズムを実装
    /// これで解けないものはかなり技巧的な技術が必要になる
    pub fn num_search(&mut self) -> bool {
        let flag = AtomicBool::new(false);
        let suudoku_nums = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        // 1から9までの数字をそれぞれ探索する
        suudoku_nums.iter().for_each(|&x| {
            let mut search_flag = false;
            // row, col, blockで一通りにきまる
            search_flag |= self.one_line_search(x);

            // 同一ブロックで2つ以上候補があるがそれが1列に並んでいる(その列に入ることが確定するので他のブロックから削除)
            search_flag |= self.line_confirmed(x);

            // 同一ブロックでn個がnマスに入る(1,2が入るマスが2つしかない場合他の数字は入れられない)
            search_flag |= self.pseudo_confirmed(x);

            if search_flag {
                flag.store(true, Ordering::Relaxed);
            }
        });
        (0..9).into_iter().for_each(|x| {
            for y in 0..9 {
                // SUGの要素数が1つの時，確定になる
                let is_done = self.field[x][y].lock().unwrap().suggest_confirmed();
                if is_done != 0 {
                    self.remove_suggest(x, y, is_done);
                    flag.store(true, Ordering::Relaxed);
                }
            }
        });
        flag.load(Ordering::Relaxed)
    }
    fn remove_suggest(&self, x: usize, y: usize, val: usize) {
        for t in 0..9 {
            // row
            match &mut *self.field[x][t].lock().unwrap() {
                FieldElement::SUG(set) => {
                    set.remove(&val);
                    assert!(set.len() > 0);
                }
                _ => (),
            }
            // col
            match &mut *self.field[t][y].lock().unwrap() {
                FieldElement::SUG(set) => {
                    set.remove(&val);
                    assert!(set.len() > 0);
                }
                _ => (),
            }
            // block
            let blockind = xy_to_block_ind(x, y);
            let (lx, ly) = BLOCK_LEFT_UPPER_CORNER[blockind];
            for (dx, dy) in BLOCK_MOV {
                let bx = lx + dx;
                let by = ly + dy;
                match &mut *self.field[bx][by].lock().unwrap() {
                    FieldElement::SUG(set) => {
                        set.remove(&val);
                        assert!(set.len() > 0);
                    }
                    _ => (),
                }
            }
        }
    }

    /// suggestが一番小さなマスを用意する
    fn pickup_elem(&self) -> PuzzleResult<std::vec::IntoIter<(Data, usize, usize)>> {
        let mut min_len = 9;
        let mut min_x = 0;
        let mut min_y = 0;
        for (x, vec) in self.field.iter().enumerate() {
            for (y, val) in vec.iter().enumerate() {
                if let FieldElement::SUG(set) = &*val.lock().unwrap() {
                    if set.len() < min_len {
                        min_len = set.len();
                        min_x = x;
                        min_y = y;
                    }
                }
            }
        }
        let min_elem = if let FieldElement::SUG(ref set) = *self.field[min_x][min_y].lock().unwrap()
        {
            set.iter()
                .map(|val| (*val, min_x, min_y))
                .collect::<Vec<_>>()
                .into_iter()
        } else {
            panic!("software buf: it cannot reach here.");
        };
        Ok(min_elem)
    }

    /// x,y座標で指定された場所に確定の値valを入れる
    fn set(&mut self, (val, x, y): (Data, usize, usize)) {
        self.field[x][y] = Arc::new(Mutex::new(FieldElement::NUM(val)));
    }

    fn is_vacant(&self) -> bool {
        for vec in &self.field {
            for val in vec {
                if let FieldElement::NUM(_) = *val.lock().unwrap() {
                } else {
                    return true;
                }
            }
        }
        false
    }

    /// num_searchだけでうまく行かないような難しい数独は仮決めして行うしかないのでdfsを回す
    /// 仮決めする値は最もsuggestが小さいようなマスを発見してイテレータで回す
    fn dfs(&mut self) -> PuzzleResult<()> {
        let mut elems = self.pickup_elem()?.into_iter();
        loop {
            // 元の状態を保持，失敗したときに戻れるようにしておく
            let clone = self.clone();

            // 候補になるelemを選出
            let elem = match elems.next() {
                Some(x) => x,
                _ => break,
            };
            self.set(elem);

            // 探索実行
            while self.num_search() {}
            if self.is_vacant() {
                // dfsの中で仮極めが必要
                match self.dfs() {
                    Ok(x) => return Ok(x),
                    Err(_) => {
                        // 失敗したので前提が間違っていた
                        *self = clone;
                    }
                }
            } else if self.has_finished()? {
                return Ok(());
            } else {
                *self = clone;
            }
        }
        Err(PuzzleError {
            error: "not found path".to_string(),
        })
    }
    fn check_block(&self) -> PuzzleResult<bool> {
        let flag = BLOCK_LEFT_UPPER_CORNER
            .par_iter()
            .map(|(cx, cy)| {
                let mut flags = [false; 9];
                for (dx, dy) in BLOCK_MOV {
                    let x = cx + dx;
                    let y = cy + dy;
                    if let FieldElement::NUM(ref n) = *self.field[x][y].lock().unwrap() {
                        let n = *n - 1;
                        if flags[n] {
                            return Err(PuzzleError::new("check_block"));
                        }
                        flags[n] = true;
                    } else {
                        return Ok(false);
                    }
                }
                Ok(true)
            })
            .reduce(|| Ok(true), |a, b| Ok(a? & b?))?;
        Ok(flag)
    }
    fn check_row(&self) -> PuzzleResult<bool> {
        let rows = [0, 1, 2, 3, 4, 5, 6, 7, 8];
        let flag = rows
            .into_par_iter()
            .map(|row| {
                let mut flags = [false; 9];
                for col in 0..9 {
                    if let FieldElement::NUM(ref n) = *self.field[row][col].lock().unwrap() {
                        let n = *n - 1;
                        if flags[n] {
                            return Err(PuzzleError::new("check_row"));
                        }
                        flags[n] = true;
                    } else {
                        return Ok(false);
                    }
                }
                Ok(true)
            })
            .reduce(|| Ok(true), |a, b| Ok(a? & b?))?;
        Ok(flag)
    }
    fn check_column(&self) -> PuzzleResult<bool> {
        let cols = [0, 1, 2, 3, 4, 5, 6, 7, 8];
        let flag = cols
            .into_par_iter()
            .map(|col| {
                let mut flags = [false; 9];
                for row in 0..9 {
                    if let FieldElement::NUM(ref n) = *self.field[row][col].lock().unwrap() {
                        let n = *n - 1;
                        if flags[n] {
                            return Err(PuzzleError::new("check_row"));
                        }
                        flags[n] = true;
                    } else {
                        return Ok(false);
                    }
                }
                Ok(true)
            })
            .reduce(|| Ok(true), |a, b| Ok(a? & b?))?;
        Ok(flag)
    }
}

impl Solver for SudokuSolver {
    // fn run(&mut self) -> Result<(), PuzzleError> {
    //     Ok(())
    // }
    // fn run(&mut self) -> PuzzleResult<()>;
    fn has_finished(&self) -> PuzzleResult<bool> {
        Ok(self.check_row()? && self.check_column()? && self.check_block()?)
    }
    fn search(&mut self) -> PuzzleResult<()> {
        if self.num_search() {
            Ok(())
        } else {
            self.dfs()
            // Ok(())
        }
    }
}
impl Display for FieldElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FieldElement::NUM(x) => x,
                FieldElement::SUG(_) => &0,
            }
        )
    }
}

impl Display for SudokuSolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str: String = "".to_string();
        for vec in &self.field {
            for val in vec {
                str += &format!("{:?}", val.lock().unwrap())
            }
            str += "\n";
        }
        write!(f, "{}", str)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn easy_test() {
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
        while !sudoku.has_finished().unwrap() {
            sudoku.search().unwrap();
        }

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
        assert!(sudoku == ans);
    }
    #[test]
    fn hard_test() {
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
        sudoku.run().unwrap();

        assert!(sudoku.has_finished().unwrap());
    }
}
