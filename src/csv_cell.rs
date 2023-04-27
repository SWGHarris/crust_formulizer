use core::fmt;
use std::{collections::HashMap, cmp};
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;


#[derive(Debug, Clone)]
pub struct CSVCell {
    pub value: CellValue,
    pub position: CellPosition,
}

// Returns a String of the csv representation for the given cells
pub fn csv_cells_to_grid(cells: &Vec<CSVCell>) -> String {
    let mut grid: HashMap<u32, HashMap<u32, CellValue>> = HashMap::new();
    let mut max_row: u32 = 0;
    let mut max_col:u32 = 0;
    for cell in cells {
        if !grid.contains_key(&cell.position.row) {
            grid.insert(cell.position.row, HashMap::new());
        }
        if grid[&cell.position.row].contains_key(&cell.position.col) {
            panic!("csv grid cannot contain overlapping CSVCells")
        }
        let x = cell.position.row;
        let y = cell.position.col;
        max_row = cmp::max(x, max_row);
        max_col = cmp::max(y, max_col);
        grid.get_mut(&x).map(|val| val.insert(y, cell.value.clone()));
    }

    let mut grid_string = String::new();
    for row in 0..(max_row + 1) {
        for col in 0..(max_col + 1) {
            if grid.contains_key(&row) && grid[&row].contains_key(&col) {
                let val = &grid[&row][&col].to_string();
                grid_string.push_str(val);
            }
            if col != max_col {
                grid_string.push(',');
            }
        }
        grid_string.push('\n');
    }
    grid_string
}


#[derive(Debug, Clone)]
pub struct CellPosition {
    pub row: u32,
    pub col: u32,
}

impl CellPosition {
    pub fn to_fixed(&self) -> Self {
        CellPosition {
            row: self.row,
            col: self.col,
        }
    }
}

impl fmt::Display for CellPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cell_ref = CellExpr::Ref(
            CellRef {
                pos: self.clone(), 
                fix_row: false, 
                fix_col: false }
            );
        let result: String = cell_ref.to_string();
        write!(f, "{}", result)
    }
}

#[derive(Debug, Clone)]
pub struct CellArray {
    from: CellRef,
    to: CellRef,
}

impl CellArray {
    pub fn new(from: CellRef, to: CellRef) -> Self {
        if from.pos.col != to.pos.col && from.pos.row != to.pos.row {
            panic!("CellArray: either 'from' or 'to' must have same value");
        }
        CellArray { from, to }
    }

    pub fn len(&self) -> u32 {
        if self.from.pos.col != self.to.pos.col {
            self.to.pos.col - self.from.pos.col + 1
        } else {
            self.to.pos.row - self.from.pos.row + 1
        }
    }
}

impl fmt::Display for CellArray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res = String::new();
        res.push_str(&self.from.to_string());
        res.push_str(":");
        res.push_str(&self.to.to_string());
        write!(f, "{}", res)
    }
}

#[derive(Debug, Clone)]
pub enum CellValue {
    Str(String),
    Expr(CellExpr),
    Empty,
}

impl fmt::Display for CellValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res = String::new();
        match &self {
            CellValue::Str(str) => res.push_str(str),
            CellValue::Empty => (),
            CellValue::Expr(expr) => {
                res.push_str("=");
                res.push_str(&expr.to_string())
            }
        }
        write!(f, "{}", res)
    }
}

#[derive(Debug,Clone)]
pub struct CellRef {
    pub pos: CellPosition,
    pub fix_row: bool,
    pub fix_col: bool
}

impl fmt::Display for CellRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res = String::new();
        if self.fix_col {
            res.push('$');
        }
        res.push_str(&to_csv_col(self.pos.col));
        if self.fix_row {
            res.push('$');
        }
        res.push_str(&(self.pos.row + 1).to_string());
        write!(f, "{}", res)
    }
}

#[derive(Debug, Clone)]
pub enum CellExpr {
    BinaryOp(BinOp, Box<CellExpr>, Box<CellExpr>),
    Ref(CellRef),
    Sum(CellArray),
    SumProduct(CellArray, CellArray),
    Number(Decimal),
    Percentage(Decimal), // 10% == 0.1
}

impl fmt::Display for CellExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res = String::new();
        match self {
            CellExpr::BinaryOp(op, left, right) => {
                res.push('(');
                res.push_str(&left.as_ref().to_string());
                res.push_str(&op.to_string());
                res.push_str(&right.as_ref().to_string());
                res.push(')');
            }
            CellExpr::Ref(cell_ref) => {
                res.push_str(&cell_ref.to_string());
            }
            CellExpr::Sum(array) => {
                res.push_str("SUM(");
                res.push_str(&array.to_string());
                res.push(')');
            }
            CellExpr::SumProduct(left, right) => {
                if left.len() != right.len() {
                    panic!("SumProduct: CellArrays cannot be if different length.");
                }
                res.push_str("SUMPRODUCT(");
                res.push_str(&left.to_string());
                res.push(',');
                res.push_str(&right.to_string());
                res.push(')');
            }
            CellExpr::Number(x) => res.push_str(&x.round_dp(3).to_string()),
            CellExpr::Percentage(val) => {
                let p = val * dec!(100);
                res.push_str(&p.round_dp(3).to_string());
                res.push('%');
            }
        }
        write!(f, "{}", res)
    }
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Div,
    Mult,
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res = String::new();
        match &self {
            BinOp::Add => res.push('+'),
            BinOp::Sub => res.push('-'),
            BinOp::Div => res.push('/'),
            BinOp::Mult => res.push('*'),
        }
        write!(f, "{}", res)
    }
}

// takes row, col as u32 and return cell label
// takes u32 and returns the column label
// ex: 4 -> D
// ex: (4 + 26) -> DD
fn to_csv_col(col: u32) -> String {
    let c: char = (b'A' + ((col % 26) as u8)) as char;
    let n = (col / 26) + 1;
    let mut col_label = String::new();
    for _ in 0..n {
        col_label.push(c);
    }
    col_label
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    const CP_1: CellRef = CellRef {
        pos: CellPosition{row: 100, col: 29},
        fix_row: false,
        fix_col: true,
    };
    const CP_2: CellRef = CellRef {
        pos: CellPosition{row: 200, col: 29},
        fix_row: false,
        fix_col: true,
    };
    const CP_3: CellRef = CellRef {
        pos: CellPosition{row: 100, col: 39},
        fix_row: false,
        fix_col: true,
    };
    const CP_4: CellRef = CellRef {
        pos: CellPosition{row: 200, col: 39},
        fix_row: false,
        fix_col: true,
    };

    #[test]
    fn test_cell_position() {
        assert_eq!(CP_1.to_string(), String::from("$DD101"));
    }

    #[test]
    fn test_cell_array() {
        let ca = CellArray::new(CP_1, CP_2);
        assert_eq!(ca.to_string(), String::from("$DD101:$DD201"));
    }

    #[test]
    #[should_panic]
    fn test_cell_array_panic() {
        CellArray::new(CP_1, CP_4);
    }

    #[test]
    fn test_sum() {
        let cv: CellValue = CellValue::Expr(CellExpr::Sum(CellArray::new(CP_1, CP_2)));
        assert_eq!(cv.to_string(), "=SUM($DD101:$DD201)")
    }

    #[test]
    fn test_sumproduct() {
        let cv: CellValue = CellValue::Expr(CellExpr::SumProduct(
            CellArray::new(CP_1, CP_2),
            CellArray::new(CP_3, CP_4),
        ));
        assert_eq!(cv.to_string(), "=SUMPRODUCT($DD101:$DD201,$NN101:$NN201)");
    }

    #[test]
    fn test_binop() {
        let e1 = CellExpr::Number(dec!(3));
        let e2: CellExpr = CellExpr::Percentage(dec!(0.1234567));
        let e3 = CellExpr::BinaryOp(BinOp::Mult, Box::new(e1), Box::new(e2));
        assert_eq!(e3.to_string(), "(3*12.3456700%)");
        let e4: CellExpr =
            CellExpr::SumProduct(CellArray::new(CP_1, CP_2), CellArray::new(CP_3, CP_4));
        let e5: CellExpr = CellExpr::BinaryOp(BinOp::Add, Box::new(e3), Box::new(e4));
        assert_eq!(
            e5.to_string(),
            "((3*12.3456700%)+SUMPRODUCT($DD101:$DD201,$NN101:$NN201))"
        );
    }
}
