use std::borrow::Borrow;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashSet;

use super::PuzzleResult;
use super::Solver;

/// 左上のブロックを基準にピースを配置する
/// ------>x
/// |
/// ↓
/// y
#[derive(Debug, PartialEq, Eq, Hash)]
struct Block {
    block: Vec<(i32, i32)>,
}
impl Block {
    /// input str is like this
    /// 010\n111\010 -> ("010111010", 3,3)
    fn new(str: &str, l: i32, h: i32) -> Self {
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

    fn flip(&self) -> Self {
        //
        let mut flip_block = Vec::new();
        flip_block.push((0, 0));
        for &(x, y) in &self.block {
            flip_block.push((-x, y));
        }
        Block::normalize(flip_block)
    }
    /// num*90度の回転を(0,0)基準で行う
    fn rotate(&self, num: u8) -> Self {
        //
        let mut rotate_block = Vec::new();
        rotate_block.push((0, 0));
        for &(mut x, mut y) in &self.block {
            for _ in 1..=num {
                x = y;
                y = -x;
            }
            rotate_block.push((x, y));
        }
        Block::normalize(rotate_block)
    }

    /// 左上が(0.0)となるように調整, (0,0)は削除する
    fn normalize(mut block: Vec<(i32, i32)>) -> Self {
        let mut minx = 0;
        let mut miny = 0;
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
#[derive(Debug)]
enum TargetType {
    ROTATE,
    FLIP,
    ROTATEFLIP,
}
#[derive(Debug, Default)]
struct TargetBlock {
    block: Vec<Block>,
    id: char,
    used: RefCell<bool>,
}

impl TargetBlock {
    fn new(str: &str, l: i32, h: i32, id: u32, targettype: TargetType) -> Self {
        let block = Block::new(str, l, h);
        match targettype {
            ROTATE => {
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
            FLIP => {
                let mut targetblock = HashSet::new();
                targetblock.insert(block.flip());
                targetblock.insert(block);
                TargetBlock {
                    block: targetblock.into_iter().collect(),
                    id: char::from_u32(id).unwrap(),
                    ..Default::default()
                }
            }
            ROTATEFLIP => {
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
    pre_l: RefCell<usize>,
    pre_h: RefCell<usize>,
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
            ("11111000", 4, 1),
            ("111100100", 3, 3),
            ("11111", 5, 1),
        ]
        .iter()
        .enumerate()
        .map(|(ind, &(str, x, y))| TargetBlock::new(str, x, y, ind as u32, TargetType::ROTATEFLIP))
        .collect::<Vec<_>>();

        let field = RefCell::new(vec![vec![None; fieldh]; fieldh]);
        let pre_l = Default::default();
        let pre_h = Default::default();

        PentominoSolver {
            blocks,
            field,
            pre_l,
            pre_h,
        }
    }
    pub fn check(&self, block: &Block) -> bool {
        let (h, l) = (self.pre_h.borrow_mut(), self.pre_l.borrow_mut());
        for &(dx, dy) in &block.block {
            let nh = *h as i32 + dy;
            let nl = *l as i32 + dx;
            let field = self.field.borrow();
            if nh >= 0 && (nh as usize) < field.len() && nl >= 0 && (nl as usize) < field[0].len() {
                if let Some(_) = self.field.borrow()[nh as usize][nl as usize] {
                    return false;
                }
            }
        }
        true
    }
    fn set_h_l(&self, h: usize, l: usize) -> (usize, usize) {
        let pre = (*self.pre_h.borrow(), *self.pre_l.borrow());
        *self.pre_h.borrow_mut() = h;
        *self.pre_l.borrow_mut() = l;
        pre
    }
    pub fn place(&self, block: &Block, id: char) -> (usize, usize) {
        let (mut h, mut l) = (self.pre_h.borrow_mut(), self.pre_l.borrow_mut());
        for &(dx, dy) in &block.block {
            let nh = (*h as i32 + dy) as usize;
            let nl = (*l as i32 + dx) as usize;
            let mut field = self.field.borrow_mut();
            (*field)[nh][nl] = Some(id);
            if *h == nh && *l > nl {
                *h = nh;
                *l = nh;
            }
        }
        self.set_h_l(*h, *l)
    }
    pub fn place_back(&self, block: &Block, h: usize, l: usize) {}
    pub fn run_all(&self) {
        // 左上に置くブロックを探す。
        // 条件を満たすか確認。
        // 満たしたら次の探索
        // ダメなら同一ブロックの他の形式をおく
        // 一つ設けなかったらflagがFalseの違うブロックで確かめる
        // 全てでダメだったら前提が間違い
        for block in &self.blocks {
            if *block.used.borrow() {
                continue;
            }
            *block.used.borrow_mut() = true;
            for one_kind_block in &block.block {
                if self.check(one_kind_block) {
                    let (h, l) = self.place(one_kind_block, block.id);
                    self.run_all();
                    self.place_back(one_kind_block, h, l);
                }
            }
            *block.used.borrow_mut() = false;
        }
    }
}

impl Solver for PentominoSolver {
    fn has_finished(&self) -> PuzzleResult<bool> {
        Ok(true)
    }
    fn search(&mut self) -> PuzzleResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::PentominoSolver;

    #[test]
    fn test() {
        // let solver = PentominoSolver::new();
        // println!("{:?}", solver);
    }
}
