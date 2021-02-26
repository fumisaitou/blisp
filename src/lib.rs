//! # BLisp
//!
//! BLisp is a well typed Lisp like programming language which adopts effect
//! system for no_std environments.
//! BLisp supports higher order RPCs like higher order functions
//! of functional programing languages.
//!
//! This repository provides only a library crate.
//! Please see [blisp-repl](https://github.com/ytakano/blisp-repl) to use BLisp,
//! or [baremetalisp](https://github.com/ytakano/baremetalisp) which is a toy OS.
//!
//! [Homepage](https://ytakano.github.io/blisp/) is here.
//!
//! ## Examples
//!
//! ### Simple Eval
//! ```
//! let code = "
//! (export factorial (n) (Pure (-> (Int) Int))
//!     (factorial' n 1))
//!
//! (defun factorial' (n total) (Pure (-> (Int Int) Int))
//!     (if (<= n 0)
//!         total
//!         (factorial' (- n 1) (* n total))))";
//!
//! let exprs = blisp::init(code).unwrap();
//! let ctx = blisp::typing(&exprs).unwrap();
//! let expr = "(factorial 10)";
//! for result in blisp::eval(expr, &ctx).unwrap() {
//!    println!("{}", result.unwrap());
//! }
//! ```
//!
//! ### Foreign Function Interface
//!
//! ```
//! use blisp;
//! use num_bigint::BigInt;
//!
//! let expr = "
//! (export callback (x y z)
//!     (IO (-> (Int Int Int) (Option Int)))
//!     (call-rust x y z))";
//! let exprs = blisp::init(expr).unwrap();
//! let mut ctx = blisp::typing(&exprs).unwrap();
//!
//! let fun = |x: &BigInt, y: &BigInt, z: &BigInt| {
//!     let n = x * y * z;
//!     println!("n = {}", n);
//!     Some(n)
//! };
//! ctx.set_callback(Box::new(fun));
//!
//! let e = "(callback 100 2000 30000)";
//! blisp::eval(e, &ctx);
//! ```
//!
//! ## Features
//!
//! - Algebraic data type
//! - Generics
//! - Hindley–Milner based type inference
//! - Effect system to separate side effects from pure functions
//! - Big integer
//! - Supporting no_std environments

#![no_std]

#[macro_use]
extern crate alloc;

use alloc::collections::linked_list::LinkedList;
use alloc::string::String;

pub mod parser;
pub mod runtime;
pub mod semantics;

const FILE_ID_PRELUD: usize = 0;
const FILE_ID_USER: usize = 1;
pub(crate) const FILE_ID_EVAL: usize = 2;

/// indicate a position of file
#[derive(Debug, Clone, Copy)]
pub struct Pos {
    pub file_id: usize, // file identifier, 0 is prelude.lisp
    pub line: usize,    // line number, 0 origin
    pub column: usize,  // column number, 0 origin
}

/// error message
#[derive(Debug)]
pub struct LispErr {
    pub msg: String,
    pub pos: Pos,
}

impl LispErr {
    fn new(msg: String, pos: Pos) -> LispErr {
        LispErr { msg: msg, pos: pos }
    }
}

/// initialize BLisp with code
///
/// # Example
///
/// ```
/// let code = "(export factorial (n) (Pure (-> (Int) Int))
///    (if (<= n 0)
///        1
///        (* n (factorial (- n 1)))))";
///
/// blisp::init(code).unwrap();
/// ```
pub fn init(code: &str) -> Result<LinkedList<parser::Expr>, LispErr> {
    let prelude = include_str!("prelude.lisp");
    let mut ps = parser::Parser::new(prelude, FILE_ID_PRELUD);
    let mut exprs = match ps.parse() {
        Ok(e) => e,
        Err(e) => {
            let msg = format!("Syntax Error: {}", e.msg);
            return Err(LispErr::new(msg, e.pos));
        }
    };

    let mut ps = parser::Parser::new(code, FILE_ID_USER);
    match ps.parse() {
        Ok(mut e) => {
            exprs.append(&mut e);
            Ok(exprs)
        }
        Err(e) => {
            let msg = format!("Syntax Error: {}", e.msg);
            Err(LispErr::new(msg, e.pos))
        }
    }
}

/// perform type checking and inference
///
/// # Example
///
/// ```
/// let code = "(export factorial (n) (Pure (-> (Int) Int))
///    (if (<= n 0)
///        1
///        (* n (factorial (- n 1)))))";
///
/// let exprs = blisp::init(code).unwrap();
/// blisp::typing(&exprs).unwrap();
/// ```
pub fn typing(exprs: &LinkedList<parser::Expr>) -> Result<semantics::Context, LispErr> {
    match semantics::exprs2context(exprs) {
        Ok(c) => Ok(c),
        Err(e) => {
            let msg = format!("Typing Error: {}", e.msg);
            Err(LispErr::new(msg, e.pos))
        }
    }
}

/// evaluate an expression
///
/// # Example
///
/// ```
/// let code = "(export factorial (n) (Pure (-> (Int) Int))
///    (if (<= n 0)
///        1
///        (* n (factorial (- n 1)))))";
///
/// let exprs = blisp::init(code).unwrap();
/// let ctx = blisp::typing(&exprs).unwrap();
/// let expr = "(factorial 30)";
/// for result in blisp::eval(expr, &ctx).unwrap() {
///    println!("{}", result.unwrap());
/// }
/// ```
pub fn eval(
    code: &str,
    ctx: &semantics::Context,
) -> Result<LinkedList<Result<String, String>>, LispErr> {
    runtime::eval(code, ctx)
}

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(test)]
mod tests {
    use crate::{eval, init, semantics, typing};

    fn eval_result(code: &str, ctx: &semantics::Context) {
        for r in eval(code, &ctx).unwrap() {
            println!("{} -> {}", code, r.unwrap());
        }
    }

    #[test]
    fn ops() {
        let exprs = init("").unwrap();
        let ctx = typing(&exprs).unwrap();
        eval_result("(+ 0x10 0x20)", &ctx);
        eval_result("(+ 0b111 0b101)", &ctx);
        eval_result("(+ 0o777 0o444)", &ctx);
        eval_result("(+ 10 20)", &ctx);
        eval_result("(pow 10 20)", &ctx);
        eval_result("(band 1 0)", &ctx);
        eval_result("(band 1 1)", &ctx);
        eval_result("(bor 1 0)", &ctx);
        eval_result("(bor 1 1)", &ctx);
        eval_result("(bxor 1 0)", &ctx);
        eval_result("(bxor 1 1)", &ctx);
        eval_result("(sqrt 16)", &ctx);
        eval_result("(sqrt -1)", &ctx);
    }

    #[test]
    fn lambda() {
        let expr = "(export lambda-test (f)
    (Pure (-> ((Pure (-> (Int Int) Int))) Int))
    (f 10 20))
";
        let exprs = init(expr).unwrap();
        let ctx = typing(&exprs).unwrap();
        let e = "(lambda-test (lambda (x y) (* x y)))";
        eval_result(e, &ctx);

        let e = "(lambda-test +)";
        eval_result(e, &ctx);
    }

    #[test]
    fn list() {
        let expr = "
(export head (x) (Pure (-> ('(Int)) (Option Int)))
    (match x
        ((Cons n _) (Some n))
        (_ None)))

(export tail (x) (Pure (-> ('(Int)) (Option Int)))
    ; match expression
    (match x
        (Nil None)
        ((Cons n Nil) (Some n))
        ((Cons _ l) (tail l))))
";
        let exprs = init(expr).unwrap();
        let ctx = typing(&exprs).unwrap();

        let e = "(head '(30 40 50))";
        eval_result(e, &ctx);

        let e = "(tail '(30 40 50))";
        eval_result(e, &ctx);
    }

    #[test]
    fn tuple() {
        let expr = "(export first (x) (Pure (-> ([Int Bool]) Int))
    (match x
        ([n _] n)))
";
        let exprs = init(expr).unwrap();
        let ctx = typing(&exprs).unwrap();
        let e = "(first [10 false])";
        eval_result(e, &ctx);
    }

    #[test]
    fn prelude() {
        let expr = "
(export factorial (n) (Pure (-> (Int) Int))
    (fact n 1))

(defun fact (n total) (Pure (-> (Int Int) Int))
    (if (<= n 0)
        total
        (fact (- n 1) (* n total))))";
        let exprs = init(expr).unwrap();
        let ctx = typing(&exprs).unwrap();

        let e = "(Some 10)";
        eval_result(e, &ctx);

        let e = "(car '(1 2 3))";
        eval_result(e, &ctx);

        let e = "(cdr '(1 2 3))";
        eval_result(e, &ctx);

        let e = "(map (lambda (x) (* x 2)) '(8 9 10))";
        eval_result(e, &ctx);

        let e = "(fold + 0 '(1 2 3 4 5 6 7 8 9))";
        eval_result(e, &ctx);

        let e = "(factorial 2000)";
        eval_result(e, &ctx);
    }

    #[test]
    fn callback() {
        let expr = "
(export callback (x y z) (IO (-> (Int Int Int) (Option Int)))
    (call-rust x y z))";
        let exprs = init(expr).unwrap();
        let mut ctx = typing(&exprs).unwrap();

        use num_bigint::BigInt;
        use std::boxed::Box;
        let fun = |x: &BigInt, y: &BigInt, z: &BigInt| {
            let n = x * y * z;
            println!("n = {}", n);
            Some(n)
        };
        ctx.set_callback(Box::new(fun));

        let e = "(callback 100 2000 30000)";
        eval_result(e, &ctx);
    }
}
