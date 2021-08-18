enum FieldElement {
    SUG(Vector<usize>),
    NUM(usize),
    NONE,
}
impl FieldElement {
    pub fn new_num(x:usize) -> FieldElement{
        FieldElement::NUM(x)
    }
    pub fn confirm_value(&self) -> FieldElement {
        match self  => {
            FieldElement::NUM(n) => {
                NUM(n)
            },
            FieldElement::NONE => {
                NONE
            },
            FieldElement::SUG(vec) => {
                if vec.size == 1 {
                    NUM(vec[0])
                }else{
                    NONE
                }
            }
            
        }

    }
}

struct Field {
    field: Vector<Vector<FieldElement>>,
}

impl Field {
    pub fn new(string: str) {}
    pub fn run(self) {}
    pub fn search_num(mut &self, x: usize, y: usize) {}
    fn search_row(&self, x: usize) {}
    fn search_column(&self, y: usize) {}
    fn search_block(&self, x: usize, y: usize) {}
    pub fn dfs(mut &self) {}
}
impl FieldElement {}
