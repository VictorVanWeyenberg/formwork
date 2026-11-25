use formwork::formwork;

#[formwork]
pub struct Inner {
    #[slot]
    pub name: String,
}

#[formwork(traverse = "bfs")]
pub enum EnumWithBfs {
    A(Inner),
}

#[formwork(traverse = "dfs")]
pub enum EnumWithDfs {
    A(Inner),
}

fn main() {}
