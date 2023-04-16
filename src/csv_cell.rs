use core::fmt;
use rust_decimal::prelude::*;

#[derive(Debug)]
pub struct CSVCell {
    pub value: CellValue,
    pub position: CellPosition
}

impl CSVCell {
    fn new(row: u32, col: u32, fix_row: bool, fix_col: bool, value: CellValue) -> Self {
        CSVCell {
            value,
            position: CellPosition {row, col, fix_row, fix_col}
        }
    }
}

#[derive(Debug)]
struct CellPosition {
    row: u32,
    col: u32,
    fix_row: bool,
    fix_col: bool
}

impl fmt::Display for CellPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result: String = Default::default();
        let fix = String::from("$");
        if self.fix_col {result.push_str(&fix);}
        result.push_str(&to_csv_col(self.col));
        if self.fix_row {result.push_str(&fix);}
        result.push_str(&(self.row + 1).to_string());
        write!(f, "{}", result)
    }
}

// dim refers to the direction of the array.
#[derive(Debug)]
struct CellArray {
    from: CellPosition,
    to: CellPosition,
    dim: CellArrayDim
}

#[derive(Debug, PartialEq, Eq)]
enum CellArrayDim {
    Row,
    Col
}

impl CellArray {
    fn new(from: CellPosition, to: CellPosition, dim: CellArrayDim) -> Self {
        if from.col != to.col && dim == CellArrayDim::Col 
            || from.row != to.row && dim == CellArrayDim::Row {
            panic!("CellArray: from/to must have same {:#?} values", dim);
        }
        CellArray { from, to, dim}
    }

    fn len(&self) -> u32 {
        match self.dim {
            CellArrayDim::Row => {
                self.to.row - self.from.row + 1
            },
            CellArrayDim::Col => {
                self.to.col - self.from.col + 1
            }
        }
    }
}

impl fmt::Display for CellArray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res = String::default();
        res.push_str(&self.from.to_string());
        res.push_str(&String::from(":"));
        res.push_str(&self.to.to_string());
        write!(f, "{}", res)
    }
}

#[derive(Debug)]
enum CellValue {
    Str(String),
    Expr(CellExpr),
    Empty
}

impl fmt::Display for CellValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res = String::from("");
        match &self {
            CellValue::Str(str) => res.push_str(str),
            CellValue::Empty => (),
            CellValue::Expr(expr) => {
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
    CellRef(CellPosition),
    Sum(CellArray),
    SumProduct(CellArray, CellArray),
    Number(Decimal),
    Percentage(Decimal), // 10% == 0.1
}

impl fmt::Display for CellExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res = String::from("");
        match self {
            CellExpr::Binop(op, left, right) => {
                res.push_str(&left.as_ref().to_string());
                res.push_str(&op.to_string());
                res.push_str(&right.as_ref().to_string());
            }
            CellExpr::CellRef(pos) => res.push_str(&pos.to_string()),
            CellExpr::Sum(array) => {
                res.push_str(&String::from("SUM("));
                res.push_str(&array.to_string());
                res.push_str(&String::from(")"));
            },
            CellExpr::SumProduct(left,right) => {
                res.push_str(&String::from("SUMPRODUCT("));
                res.push_str(&left.to_string());
                res.push_str(&String::from(","));
                res.push_str(&right.to_string());
                res.push_str(&String::from(")"));
            },
            CellExpr::Number(x) => res.push_str(&x.to_string()),
            CellExpr::Percentage(val) => {
                res.push_str(&val.to_string());
                res.push_str(&String::from("%"));
            },
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
            Op::Add  => res.push_str(&String::from("+")),
            Op::Sub  => res.push_str(&String::from("-")),
            Op::Div  => res.push_str(&String::from("/")),
            Op::Mult => res.push_str(&String::from("*")),
        }
        write!(f, "{}", res)
    }
}

// takes row, col as u32 and return cell label
// if fix_cell then returns fixed value (e.g. $H$7))
// takes u32 and returns the column label
// ex: 4 -> D
// ex: (4 + 26) -> DD
fn to_csv_col(col: u32) -> String {
    let c: u8 = b'A' + ((col % 26) as u8);
    let n = (col / 26) + 1;
    let mut col_label = String::from("");
    for _ in 0..n {
        let c_str = (c as char).to_string();
        col_label.push_str(&c_str);
     }
     col_label
}