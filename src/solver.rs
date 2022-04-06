use std::collections::BTreeSet;

type Data = usize;

#[derive(PartialEq, Eq, Hash)]
enum FieldElement {
    SUGGEST(BTreeSet<Data>),
    NUM(Data),
    NONE,
}

impl FieldElement {
    pub fn set_n(x: Data) -> FieldElement {
        FieldElement::NUM(x)
    }
    pub fn is_confirm(&self) -> bool {
        match self {
            FieldElement::NUM(_) => true,
            FieldElement::SUGGEST(x) => x.len() == 1,
            _ => false,
        }
    }
}

struct Field {
    field: Vec<Vec<FieldElement>>,
    row: [BTreeSet<Data>; 9],
    column: [BTreeSet<Data>; 9],
}

impl Field {
    pub fn new(string: &str) {}
    pub fn run(self) {}
    fn search_column(&mut self, x: Data) {
        {
            let col = &mut self.column[x];
            for y in 0..=9 {
                let elem = &self.field[y][x];
                if let FieldElement::NUM(n) = elem {
                    col.insert(*n);
                }
            }
        }
        let col = &self.column[x];
        for y in 0..=9 {
            let elem = &mut self.field[y][x];
            if let FieldElement::SUGGEST(x) = elem {
                for c in col {
                    x.remove(c);
                }
            }
        }
    }
    fn search_row(&mut self, y: Data) {
        {
            let row = &mut self.row[y];
            for x in 0..=9 {
                let elem = &self.field[x][y];
                if let FieldElement::NUM(n) = elem {
                    row.insert(*n);
                }
            }
        }
        let row = &self.row[y];
        for x in 0..=9 {
            let elem = &mut self.field[x][y];
            if let FieldElement::SUGGEST(y) = elem {
                for r in row {
                    y.remove(r);
                }
            }
        }
    }
    fn search_block(&self, x: usize, y: usize) -> bool {
        let xb = x / 3;
        let yb = y / 3;
        let mut hash = BTreeSet::new();
        for xi in 0..3 {
            for yi in 0..3 {
                let x = xi + xb;
                let y = yi + yb;
                let elem = &self.field[y][x];
                if let FieldElement::NUM(n) = elem {
                    if hash.contains(n) {
                        return false;
                    }
                    hash.insert(n);
                }
            }
        }
        true
    }
    pub fn dfs(&mut self) {}
}
impl FieldElement {}
