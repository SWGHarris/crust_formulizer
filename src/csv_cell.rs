use core::fmt;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

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
pub struct CellPosition {
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
pub struct CellArray {
    from: CellPosition,
    to: CellPosition,
    // dim: CellArrayDim
}

impl CellArray {
    pub fn new(from: CellPosition, to: CellPosition) -> Self {
        if from.col != to.col && from.row != to.row {
            panic!("CellArray: either 'from' or 'to' must have same value");
        }
        CellArray { from, to }
    }

    pub fn len(&self) -> u32 {
        if self.from.col != self.to.col {
            self.to.col - self.from.col + 1
        } else {
            self.to.row - self.from.row + 1
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
pub enum CellValue {
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
pub enum CellExpr {
    BinaryOp(BinOp, Box<CellExpr>, Box<CellExpr>),
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
            CellExpr::BinaryOp(op, left, right) => {
                res.push('(');
                res.push_str(&left.as_ref().to_string());
                res.push_str(&op.to_string());
                res.push_str(&right.as_ref().to_string());
                res.push(')');
            }
            CellExpr::CellRef(pos) => res.push_str(&pos.to_string()),
            CellExpr::Sum(array) => {
                res.push_str(&String::from("SUM("));
                res.push_str(&array.to_string());
                res.push_str(&String::from(")"));
            },
            CellExpr::SumProduct(left,right) => {
                if left.len() != right.len() {
                    panic!("SumProduct: CellArrays cannot be if different length.");
                }
                res.push_str(&String::from("SUMPRODUCT("));
                res.push_str(&left.to_string());
                res.push_str(&String::from(","));
                res.push_str(&right.to_string());
                res.push_str(&String::from(")"));
            },
            CellExpr::Number(x) => res.push_str(&x.to_string()),
            CellExpr::Percentage(val) => {
                let p = val * dec!(100);
                res.push_str(&p.to_string());
                res.push_str(&String::from("%"));
            },
        }
        write!(f, "{}", res)
    }
}

#[derive(Debug)]
pub enum BinOp {
    Add,
    Sub,
    Div,
    Mult
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res = String::from("");
        match &self {
            BinOp::Add  => res.push_str(&String::from("+")),
            BinOp::Sub  => res.push_str(&String::from("-")),
            BinOp::Div  => res.push_str(&String::from("/")),
            BinOp::Mult => res.push_str(&String::from("*")),
        }
        write!(f, "{}", res)
    }
}

// takes row, col as u32 and return cell label
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


#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;
    use super::*;

    const CP_1: CellPosition = CellPosition { row: 100, col: 29, fix_row: false, fix_col: true };
    const CP_2: CellPosition = CellPosition { row: 200, col: 29, fix_row: false, fix_col: true };
    const CP_3: CellPosition = CellPosition { row: 100, col: 39, fix_row: false, fix_col: true };
    const CP_4: CellPosition = CellPosition { row: 200, col: 39, fix_row: false, fix_col: true };
    
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
        let cv: CellValue = CellValue::Expr(CellExpr::SumProduct(CellArray::new(CP_1, CP_2), CellArray::new(CP_3, CP_4)));
        assert_eq!(cv.to_string(), "=SUMPRODUCT($DD101:$DD201,$NN101:$NN201)");
    }

    #[test]
    fn test_binop() {
        let e1 = CellExpr::Number(dec!(3));
        let e2: CellExpr = CellExpr::Percentage(dec!(0.1234567));
        let e3 = CellExpr::BinaryOp(BinOp::Mult, Box::new(e1), Box::new(e2));
        assert_eq!(e3.to_string(), "(3*12.3456700%)");
        let e4: CellExpr = CellExpr::SumProduct(CellArray::new(CP_1, CP_2), CellArray::new(CP_3, CP_4));
        let e5: CellExpr = CellExpr::BinaryOp(BinOp::Add, Box::new(e3), Box::new(e4));
        assert_eq!(e5.to_string(), "((3*12.3456700%)+SUMPRODUCT($DD101:$DD201,$NN101:$NN201))");
    }

}