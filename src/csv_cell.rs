use core::fmt;

#[derive(Debug)]
enum CellDim {
    Row,
    Col
}

#[derive(Debug)]
pub struct CSVCell {
    pub expr: CellContent,
    pub row: u32,
    pub col: u32,
}

#[derive(Debug)]
struct CellRange {
    dim: CellDim,
    dim_first: u32,
    dim_last: u32, // inclusive
    dim_fixed: u32
}

impl fmt::Display for CellRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res = String::from("");
        write!(f, "{}", res)
    }
}

#[derive(Debug)]
enum CellContent {
    Str(String),
    Expr(CellExpr),
    Empty
}

impl fmt::Display for CellContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res = String::from("");
        match &self {
            CellContent::Str(str) => res.push_str(str),
            CellContent::Empty => (),
            CellContent::Expr(expr) => {
                res.push_str(&String::from("="));
                res.push_str(&expr.to_string())
            }
        }
        write!(f, "{}", res)
    }
}


#[derive(Debug)]
enum CellExpr {
    Binop(Op, Box<CellExpr>, Box<CellExpr>),
    CellRef(u32,u32),
    Sum(CellRange),
    SumProduct(CellRange, CellRange),
    Float(f64),
    Percentage(f64), // 10% == 0.1
    Int(u32),
}

impl fmt::Display for CellExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res = String::from("");
        match self {
            CellExpr::Binop(op, left, right) => {
                    res.push_str(&left.to_string());
                    res.push_str(&op.to_string());
                    res.push_str(&right.to_string());
            }
            CellExpr::Float(x) => res.push_str(&x.to_string()),
            CellExpr::Int(x) => res.push_str(&x.to_string()),
            CellExpr::CellRef(_) => todo!(),
            CellExpr::Sum(_) => todo!(),
            CellExpr::SumProduct(_, _) => todo!(),
            CellExpr::Percentage(_) => todo!(),
        }
        write!(f, "{}", res)
    }
}

#[derive(Debug)]
enum Op {
    Add,
    Sub,
    Div,
    Mult
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res = String::from("");
        match &self {
            Op::Add => res.push_str(&String::from("+")),
            Op::Sub => res.push_str(&String::from("-")),
            Op::Div => res.push_str(&String::from("/")),
            Op::Mult => res.push_str(&String::from("*")),
        }
        write!(f, "{}", res)
    }
}