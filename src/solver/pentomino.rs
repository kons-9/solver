use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::Display;

use super::PuzzleResult;
use super::Solver;

/// 左上のブロックを基準にピースを配置する
/// ------>x
/// |
/// ↓
/// y
#[derive(Debug, Hash)]
pub struct Block {
    block: Vec<(i32, i32)>,
}
impl Block {
    /// input str is like this
    /// 010\n111\010 -> ("010111010", 3,3)
    pub fn new(str: &str, l: i32, h: i32) -> Self {
        let mut block = Vec::new();
        let mut iter = str.chars();
        for y in 0..h {
            for x in 0..l {
                let c = iter.next().unwrap();
                if c == '1' {
                    block.push((x, y));
                }
            }
        }
        Self::normalize(block)
    }

    pub fn flip(&self) -> Self {
        //
        let mut flip_block = Vec::new();
        flip_block.push((0, 0));
        for &(x, y) in &self.block {
            flip_block.push((-x, y));
        }
        Block::normalize(flip_block)
    }
    /// num*90度の回転を(0,0)基準で行う
    pub fn rotate(&self, num: u8) -> Self {
        //
        let mut rotate_block = Vec::new();
        rotate_block.push((0, 0));
        for (x, y) in &self.block {
            let mut nx = *x;
            let mut ny = *y;
            for _ in 1..=num {
                (nx, ny) = (-ny, nx);
            }
            rotate_block.push((nx, ny));
        }
        Block::normalize(rotate_block)
    }

    /// 左上が(0.0)となるように調整, (0,0)は削除する
    fn normalize(mut block: Vec<(i32, i32)>) -> Self {
        let mut minx = std::i32::MAX;
        let mut miny = std::i32::MAX;
        let mut minind = 0;
        // 左上の座標の相対座標を求める
        for (i, &(x, y)) in block.iter().enumerate() {
            if miny > y {
                minx = x;
                miny = y;
                minind = i;
            } else if miny == y && x < minx {
                minx = x;
                miny = y;
                minind = i;
            }
        }
        for (x, y) in block.iter_mut() {
            *x -= minx;
            *y -= miny;
        }
        block.remove(minind);
        Block { block }
    }
}
impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        'outer: for i in &self.block {
            for j in &other.block {
                if i == j {
                    break 'outer;
                }
            }
            return false;
        }
        return true;
    }
}
impl Eq for Block {}
#[derive(Debug)]
pub enum TargetType {
    ROTATE,
    FLIP,
    ROTATEFLIP,
}
#[derive(Debug, Default)]
pub struct TargetBlock {
    block: Vec<Block>,
    id: char,
    used: RefCell<bool>,
}

impl TargetBlock {
    pub fn new(str: &str, l: i32, h: i32, id: u32, targettype: &TargetType) -> Self {
        let block = Block::new(str, l, h);
        match targettype {
            TargetType::ROTATE => {
                let mut targetblock = HashSet::new();
                for i in 0..=3 {
                    targetblock.insert(block.rotate(i));
                }
                TargetBlock {
                    block: targetblock.into_iter().collect(),
                    id: char::from_u32(id).unwrap(),
                    ..Default::default()
                }
            }
            TargetType::FLIP => {
                let mut targetblock = HashSet::new();
                targetblock.insert(block.flip());
                targetblock.insert(block);
                TargetBlock {
                    block: targetblock.into_iter().collect(),
                    id: char::from_u32(id).unwrap(),
                    ..Default::default()
                }
            }
            TargetType::ROTATEFLIP => {
                let mut targetblock = HashSet::new();
                for i in 0..=3 {
                    targetblock.insert(block.rotate(i));
                    targetblock.insert(block.flip().rotate(i));
                }
                TargetBlock {
                    block: targetblock.into_iter().collect(),
                    id: char::from_u32(id).unwrap(),
                    ..Default::default()
                }
            }
        }
    }
}

type Field = Vec<Vec<Option<char>>>;
#[derive(Debug)]
pub struct PentominoSolver {
    blocks: Vec<TargetBlock>,
    field: RefCell<Field>,
}
impl Display for PentominoSolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for vec in &*self.field.borrow() {
            for c in vec {
                if let Some(x) = c {
                    write!(f, "{}", x)?;
                } else {
                    write!(f, "0")?;
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl PentominoSolver {
    pub fn new(fieldh: usize, fieldl: usize) -> Self {
        let blocks = [
            ("010111010", 3, 3),
            ("111101", 3, 2),
            ("110011001", 3, 3),
            ("110011010", 3, 3),
            ("110010011", 3, 3),
            ("111110", 2, 3),
            ("11100011", 4, 2),
            ("11110100", 4, 2),
            ("111010010", 3, 3),
            ("11111000", 4, 2),
            ("111100100", 3, 3),
            ("11111", 5, 1),
        ]
        .iter()
        .enumerate()
        .map(|(ind, &(str, x, y))| {
            TargetBlock::new(str, x, y, (ind + 100) as u32, &TargetType::FLIP)
        })
        .collect::<Vec<_>>();

        let field = RefCell::new(vec![vec![None; fieldl]; fieldh]);

        PentominoSolver { blocks, field }
    }
    pub fn from_vec(
        vec: Vec<(&str, i32, i32)>,
        targettype: TargetType,
        fieldh: usize,
        fieldl: usize,
    ) -> Self {
        let blocks = vec
            .iter()
            .enumerate()
            .map(|(ind, &(str, x, y))| TargetBlock::new(str, x, y, ind as u32, &targettype))
            .collect::<Vec<_>>();

        let field = RefCell::new(vec![vec![None; fieldl]; fieldh]);

        PentominoSolver { blocks, field }
    }
    #[inline]
    fn check(&self, block: &Block, h: i32, l: i32) -> bool {
        let field = self.field.borrow();
        #[cfg(test)]
        {
            assert!(
                h >= 0 && (h as usize) < field.len() && l >= 0 && (l as usize) < field[0].len()
            );

            if let Some(_) = self.field.borrow()[h as usize][l as usize] {
                panic!();
            }
        }

        for &(dx, dy) in &block.block {
            let nh = h + dy;
            let nl = l + dx;
            if nh >= 0 && (nh as usize) < field.len() && nl >= 0 && (nl as usize) < field[0].len() {
                if let Some(_) = self.field.borrow()[nh as usize][nl as usize] {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
    #[inline]
    fn place(&self, block: &Block, id: char, h: i32, l: i32) {
        let mut field = self.field.borrow_mut();
        (*field)[h as usize][l as usize] = Some(id);
        for &(dx, dy) in &block.block {
            // checkが住んでいるものなのでusizeに置き換え可能
            let nh = (h + dy) as usize;
            let nl = (l + dx) as usize;
            (*field)[nh][nl] = Some(id);
        }
    }
    #[inline]
    fn place_back(&self, block: &Block, h: i32, l: i32) {
        let mut field = self.field.borrow_mut();
        (*field)[h as usize][l as usize] = None;
        for &(dx, dy) in &block.block {
            let nh = (h + dy) as usize;
            let nl = (l + dx) as usize;
            (*field)[nh][nl] = None;
        }
    }
    pub fn run_all(&self) -> u32 {
        self._run_all(0, 0)
    }
    fn find_upper_left(&self, pre_h: i32, pre_l: i32) -> Option<(i32, i32)> {
        let field = self.field.borrow();
        for j in pre_l as usize..field[0].len() {
            if let None = field[pre_h as usize][j] {
                return Some((pre_h, j as i32));
            }
        }
        for i in pre_h as usize + 1..field.len() {
            for j in 0..field[0].len() {
                if let None = field[i][j] {
                    return Some((i as i32, j as i32));
                }
            }
        }
        None
    }
    fn _run_all(&self, pre_h: i32, pre_l: i32) -> u32 {
        // 左上に置くブロックを探す。
        // 条件を満たすか確認。
        // 満たしたら次の探索
        // ダメなら同一ブロックの他の形式をおく
        // 一つ設けなかったらflagがFalseの違うブロックで確かめる
        // 全てでダメだったら前提が間違い
        let (h, l) = match self.find_upper_left(pre_h, pre_l) {
            Some(x) => x,
            None => {
                // ない時は全部埋まっているということ
                return 1;
            }
        };
        let mut cnt = 0;
        for targetblock in &self.blocks {
            if *targetblock.used.borrow() {
                continue;
            }
            *targetblock.used.borrow_mut() = true;
            for one_kind_block in &targetblock.block {
                if self.check(one_kind_block, h, l) {
                    self.place(one_kind_block, targetblock.id, h, l);
                    cnt += self._run_all(h, l);
                    self.place_back(one_kind_block, h, l);
                }
            }
            *targetblock.used.borrow_mut() = false;
        }
        cnt
    }
}

impl Solver for PentominoSolver {
    fn has_finished(&self) -> PuzzleResult<bool> {
        let field = self.field.borrow();
        for i in &*field {
            for c in i {
                if let None = c {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
    fn search(&mut self) -> PuzzleResult<()> {
        self.run_all();
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{Block, PentominoSolver, TargetBlock, TargetType};
    /// Block tests
    #[test]
    fn pentomino_block_test() {
        let block = Block::new("110111", 3, 2);
        assert_eq!(block.flip(), Block::new("011111", 3, 2));
        assert_eq!(block.rotate(0), Block::new("110111", 3, 2));
        assert_eq!(block.rotate(1), Block::new("111110", 2, 3));
        assert_eq!(block.rotate(2), Block::new("111011", 3, 2));
        assert_eq!(block.rotate(3), Block::new("011111", 2, 3));
        assert_eq!(block.rotate(4), Block::new("110111", 3, 2));
    }

    #[test]
    fn pentomino_targetblock_test() {
        let targetblock = TargetBlock::new("010111010", 3, 3, 100, &TargetType::ROTATEFLIP);
        println!("{:?}", targetblock);
    }

    #[test]
    fn pentomino_test() {
        let solver = PentominoSolver::new(6, 10);
        let cnt = solver.run_all();
        assert_eq!(cnt, 2339);
    }
    #[test]
    fn pentomino_place_test() {
        let solver = PentominoSolver::new(6, 10);
        let block = Block::new("110111", 3, 2);
        solver.place(&block, char::from_u32(100).unwrap(), 0, 0);
        eprintln!("solver: \n{}", solver);
        solver.place(&block, char::from_u32(100).unwrap(), 3, 0);
        eprintln!("solver: \n{}", solver);
        solver.place_back(&block, 3, 0);
        eprintln!("solver: \n{}", solver);
        solver.place(&block, char::from_u32(100).unwrap(), 3, 3);
        eprintln!("solver: \n{}", solver);
        solver.place_back(&block, 0, 0);
        eprintln!("solver: \n{}", solver);
    }
    #[test]
    #[should_panic]
    fn pentomino_check_panic_test() {
        let solver = PentominoSolver::new(6, 10);
        let block = Block::new("110011011", 3, 3);
        solver.place(&block, char::from_u32(100).unwrap(), 0, 0);
        let _ = solver.check(&block, 0, 0);
        let _ = solver.check(&block, 0, 1);
        let _ = solver.check(&block, 1, 1);
        let _ = solver.check(&block, 1, 2);
        let _ = solver.check(&block, 2, 2);
    }
    #[test]
    fn pentomino_check_test() {
        let solver = PentominoSolver::new(6, 10);
        let block = Block::new("110011001", 3, 3);
        solver.place(&block, char::from_u32(100).unwrap(), 0, 0);
        let n = solver.check(&block, 1, 0);
        assert_eq!(n, false);
        let n = solver.check(&block, 0, 2);
        assert_eq!(n, true);
        eprintln!("{}", solver);
        let n = solver.check(&block, 2, 0);
        assert_eq!(n, true);
        let n = solver.check(&block, 0, 7);
        assert_eq!(n, true);
        let n = solver.check(&block, 0, 8);
        assert_eq!(n, false);
        let n = solver.check(&block, 0, 9);
        assert_eq!(n, false);
        let n = solver.check(&block, 5, 7);
        assert_eq!(n, false);
        let n = solver.check(&block, 4, 7);
        assert_eq!(n, false);
        let n = solver.check(&block, 3, 7);
        assert_eq!(n, true);
    }
    #[test]
    fn pentomino_find_upper_left() {
        let solver = PentominoSolver::new(6, 10);
        let block = Block::new("11", 1, 2);
        solver.place(&block, char::from_u32(100).unwrap(), 0, 0);
        let n = solver.find_upper_left(0, 0).unwrap();
        assert_eq!(n, (0, 1));

        let block = Block::new("11", 2, 1);
        solver.place(&block, char::from_u32(101).unwrap(), n.0, n.1);
        let n = solver.find_upper_left(0, 1).unwrap();
        assert_eq!(n, (0, 3));

        let block = Block::new("111111111", 3, 3);
        solver.place(&block, char::from_u32(102).unwrap(), n.0, n.1);
        let n = solver.find_upper_left(0, 1).unwrap();
        assert_eq!(n, (0, 6));

        let block = Block::new("1111", 4, 1);
        solver.place(&block, char::from_u32(103).unwrap(), n.0, n.1);
        let n = solver.find_upper_left(0, 1).unwrap();
        assert_eq!(n, (1, 1));

        for i in &mut *solver.field.borrow_mut() {
            for j in i {
                if let None = *j {
                    *j = char::from_u32(104);
                }
            }
        }
        let n = solver.find_upper_left(n.0, n.1);
        assert_eq!(n, None);
        println!("{}", solver);
    }
}
