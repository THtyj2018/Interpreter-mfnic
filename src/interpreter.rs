//! Interpreter

use std::{collections::HashMap, sync::Arc};

use crate::{
    lexer::{AddSubOp, CompareOp, Ident, Lexer, MulDivOp},
    parser::{ASTNode, Parser},
    InvalidToken, Real,
};

struct Function {
    incount: usize,
    fimpl: FunctionImpl,
}

enum FunctionImpl {
    Lib(fn(&[Real]) -> Real),
    User(ExprOrNum),
}

enum ExprOrNum {
    Expr(Box<Expression>),
    Num(Real),
}

impl ExprOrNum {
    fn assume_num(self) -> Real {
        match self {
            ExprOrNum::Num(real) => real,
            _ => panic!("Can't unwrap a number"),
        }
    }
}

enum Expression {
    Not(Box<Expression>),
    Neg(Box<Expression>),
    Exp(ExprOrNum, ExprOrNum),
    Mul(ExprOrNum, ExprOrNum),
    Div(ExprOrNum, ExprOrNum),
    Add(ExprOrNum, ExprOrNum),
    Sub(ExprOrNum, ExprOrNum),
    Compare(CompareOp, ExprOrNum, ExprOrNum),
    Or(ExprOrNum, ExprOrNum),
    And(ExprOrNum, ExprOrNum),
    Condition(Box<Expression>, ExprOrNum, ExprOrNum),
    Invoke(Option<Arc<Function>>, Vec<ExprOrNum>),
    Variable(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputError {
    InvalidToken(InvalidToken),
    SyntaxError { column: usize },
    RepeatVariable { ident: Ident },
    UndefinedIdentifier { ident: Ident },
    BuiltinIdentifier { ident: Ident },
    InconsistentVariablesCount { ident: Ident },
}

impl ToString for InputError {
    fn to_string(&self) -> String {
        match self {
            InputError::InvalidToken(e) => format!("{:?}", e),
            InputError::SyntaxError { column } => format!("Syntax Error at column {}", column),
            InputError::RepeatVariable { ident } => format!(
                "Repeat Variable: {}",
                String::from_utf8(ident.clone()).unwrap()
            ),
            InputError::UndefinedIdentifier { ident } => format!(
                "Undefined Identifier: {}",
                String::from_utf8(ident.clone()).unwrap()
            ),
            InputError::BuiltinIdentifier { ident } => format!(
                "Use Builtin Identifier: {}",
                String::from_utf8(ident.clone()).unwrap()
            ),
            InputError::InconsistentVariablesCount { ident } => format!(
                "Inconsistent Variables Count: {}",
                String::from_utf8(ident.clone()).unwrap()
            ),
        }
    }
}

impl From<InvalidToken> for InputError {
    fn from(e: InvalidToken) -> Self {
        InputError::InvalidToken(e)
    }
}

pub struct Interpreter {
    values: HashMap<Ident, (bool, Real)>,
    functions: HashMap<Ident, Arc<Function>>,
    parser: Option<Parser>,
    cur_ident: Ident,
    cur_variables: Vec<Ident>,
}

pub enum InputState {
    Empty,
    Incomplete,
    Assignment,
    Expression,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut itp = Interpreter {
            values: HashMap::new(),
            functions: HashMap::new(),
            parser: None,
            cur_ident: vec![],
            cur_variables: vec![],
        };
        itp.values.insert(b"_".to_vec(), (false, 0.0));
        itp.insert_builtin_value(b"pi", 3.141592653589793);
        itp.insert_builtin_value(b"e", 2.718281828459045);
        itp.insert_builtin_fn(b"abs", 1, |v| v[0].abs());
        itp.insert_builtin_fn(b"floor", 1, |v| v[0].floor());
        itp.insert_builtin_fn(b"ceil", 1, |v| v[0].ceil());
        itp.insert_builtin_fn(b"round", 1, |v| v[0].round());
        itp.insert_builtin_fn(b"sgn", 1, |v| v[0].signum());
        itp.insert_builtin_fn(b"sqrt", 1, |v| v[0].sqrt());
        itp.insert_builtin_fn(b"cbrt", 1, |v| v[0].cbrt());
        itp.insert_builtin_fn(b"sin", 1, |v| v[0].sin());
        itp.insert_builtin_fn(b"cos", 1, |v| v[0].cos());
        itp.insert_builtin_fn(b"tan", 1, |v| v[0].tan());
        itp.insert_builtin_fn(b"asin", 1, |v| v[0].asin());
        itp.insert_builtin_fn(b"acos", 1, |v| v[0].acos());
        itp.insert_builtin_fn(b"atan", 1, |v| v[0].atan());
        itp.insert_builtin_fn(b"atan2", 2, |v| v[1].atan2(v[0]));
        itp.insert_builtin_fn(b"ln", 1, |v| v[0].ln());
        itp.insert_builtin_fn(b"log", 1, |v| v[0].log10());
        itp
    }

    fn insert_builtin_value(&mut self, ident: &[u8], value: Real) {
        self.values.insert(ident.to_vec(), (true, value));
    }

    fn insert_builtin_fn(&mut self, ident: &[u8], incount: usize, f: fn(&[Real]) -> Real) {
        self.functions
            .insert(ident.to_vec(), Function::builtin(incount, f));
    }

    pub fn input(&mut self, line: &[u8]) -> Result<InputState, InputError> {
        let ts = Lexer::new(line).tokenize()?;
        let mut parser = match self.parser.take() {
            Some(parser) => parser,
            None => {
                if ts.tokens.is_empty() {
                    return Ok(InputState::Empty);
                }
                Parser::new()
            }
        };
        for (column, token) in ts.tokens {
            if !parser.action(token) {
                return Err(InputError::SyntaxError { column });
            }
        }
        if ts.complete {
            match parser.accept() {
                Some(ast) => self.translate_ast(ast),
                None => Err(InputError::SyntaxError { column: line.len() }),
            }
        } else {
            self.parser.replace(parser);
            Ok(InputState::Incomplete)
        }
    }

    pub fn last_result(&self) -> Real {
        self.values.get(&b"_".to_vec()).unwrap().1
    }

    fn translate_ast(&mut self, ast: ASTNode) -> Result<InputState, InputError> {
        match ast {
            // statement: assignment
            ASTNode::Inner(1, mut children) => match children.pop().unwrap() {
                // assignment: IDENT '=' expression
                ASTNode::Inner(3, mut children) => {
                    let expr_ast = children.pop().unwrap();
                    children.pop();
                    let ident = children.pop().unwrap().assume_leaf().assume_ident();
                    if self.is_builtin_value(&ident) {
                        return Err(InputError::BuiltinIdentifier { ident });
                    }
                    self.cur_ident.clear();
                    self.cur_variables.clear();
                    let expression = self.translate_expression(expr_ast)?;
                    self.values.insert(ident, (false, expression.assume_num()));
                    Ok(InputState::Assignment)
                }
                // assignment: IDENT ':' variable_list '=' expression
                ASTNode::Inner(4, mut children) => {
                    let expr_ast = children.pop().unwrap();
                    children.pop();
                    let variables = self.translate_variable_list(children.pop().unwrap())?;
                    for (i, var) in variables.iter().enumerate() {
                        if variables.iter().rposition(|v| v == var).unwrap() != i {
                            return Err(InputError::RepeatVariable { ident: var.clone() });
                        }
                    }
                    self.cur_variables = variables;
                    children.pop();
                    let ident = children.pop().unwrap().assume_leaf().assume_ident();
                    if self.is_builtin(&ident) {
                        return Err(InputError::BuiltinIdentifier { ident });
                    }
                    self.cur_ident = ident;
                    let expression = self.translate_expression(expr_ast)?;
                    let function = Function {
                        incount: self.cur_variables.len(),
                        fimpl: FunctionImpl::User(expression),
                    };
                    self.functions
                        .insert(self.cur_ident.clone(), Arc::new(function));
                    Ok(InputState::Assignment)
                }
                _ => unreachable!(),
            },
            // statement: expression
            ASTNode::Inner(2, mut children) => {
                self.cur_ident.clear();
                self.cur_variables.clear();
                let expression = self.translate_expression(children.pop().unwrap())?;
                self.values
                    .insert(b"_".to_vec(), (false, expression.assume_num()));
                Ok(InputState::Expression)
            }
            _ => unreachable!(),
        }
    }

    fn translate_expression(&self, ast: ASTNode) -> Result<ExprOrNum, InputError> {
        match ast {
            // expression: '(' expression ')'
            ASTNode::Inner(7, mut children) => {
                children.pop();
                self.translate_expression(children.pop().unwrap())
            }
            // expression: '!' expression
            ASTNode::Inner(8, mut children) => {
                let res = self.translate_expression(children.pop().unwrap())?;
                Ok(match res {
                    ExprOrNum::Expr(expr) => ExprOrNum::Expr(Box::new(Expression::Not(expr))),
                    ExprOrNum::Num(real) => ExprOrNum::Num(if real == 0.0 { 1.0 } else { 0.0 }),
                })
            }
            // expression: PN expression
            ASTNode::Inner(9, mut children) => {
                let res = self.translate_expression(children.pop().unwrap())?;
                let pn = children.pop().unwrap().assume_leaf().assume_pn();
                Ok(match res {
                    ExprOrNum::Expr(expr) => ExprOrNum::Expr(match pn {
                        AddSubOp::ADD => expr,
                        AddSubOp::SUB => Box::new(Expression::Neg(expr)),
                    }),
                    ExprOrNum::Num(real) => ExprOrNum::Num(match pn {
                        AddSubOp::ADD => real,
                        AddSubOp::SUB => -real,
                    }),
                })
            }
            // expression: expression '^' expression
            ASTNode::Inner(10, mut children) => {
                let ex2 = self.translate_expression(children.pop().unwrap())?;
                children.pop();
                let ex1 = self.translate_expression(children.pop().unwrap())?;
                Ok(match (ex1, ex2) {
                    (ExprOrNum::Num(r1), ExprOrNum::Num(r2)) => ExprOrNum::Num(r1.powf(r2)),
                    (ex1, ex2) => ExprOrNum::Expr(Box::new(Expression::Exp(ex1, ex2))),
                })
            }
            // expression: expression MD expression
            ASTNode::Inner(11, mut children) => {
                let ex2 = self.translate_expression(children.pop().unwrap())?;
                let md = children.pop().unwrap().assume_leaf().assume_md();
                let ex1 = self.translate_expression(children.pop().unwrap())?;
                Ok(match (ex1, ex2) {
                    (ExprOrNum::Num(r1), ExprOrNum::Num(r2)) => ExprOrNum::Num(match md {
                        MulDivOp::MUL => r1 * r2,
                        MulDivOp::DIV => r1 / r2,
                    }),
                    (ex1, ex2) => ExprOrNum::Expr(Box::new(match md {
                        MulDivOp::MUL => Expression::Mul(ex1, ex2),
                        MulDivOp::DIV => Expression::Div(ex1, ex2),
                    })),
                })
            }
            // expression: expression PN expression
            ASTNode::Inner(12, mut children) => {
                let ex2 = self.translate_expression(children.pop().unwrap())?;
                let pn = children.pop().unwrap().assume_leaf().assume_pn();
                let ex1 = self.translate_expression(children.pop().unwrap())?;
                Ok(match (ex1, ex2) {
                    (ExprOrNum::Num(r1), ExprOrNum::Num(r2)) => ExprOrNum::Num(match pn {
                        AddSubOp::ADD => r1 + r2,
                        AddSubOp::SUB => r1 - r2,
                    }),
                    (ex1, ex2) => ExprOrNum::Expr(Box::new(match pn {
                        AddSubOp::ADD => Expression::Add(ex1, ex2),
                        AddSubOp::SUB => Expression::Sub(ex1, ex2),
                    })),
                })
            }
            // expression: expression CMP expression
            ASTNode::Inner(13, mut children) => {
                let ex2 = self.translate_expression(children.pop().unwrap())?;
                let cmp = children.pop().unwrap().assume_leaf().assume_cmp();
                let ex1 = self.translate_expression(children.pop().unwrap())?;
                Ok(match (ex1, ex2) {
                    (ExprOrNum::Num(r1), ExprOrNum::Num(r2)) => ExprOrNum::Num(cmp.on(r1, r2)),
                    (ex1, ex2) => ExprOrNum::Expr(Box::new(Expression::Compare(cmp, ex1, ex2))),
                })
            }
            // expression: expression OR expression
            ASTNode::Inner(14, mut children) => {
                let ex2 = self.translate_expression(children.pop().unwrap())?;
                children.pop();
                let ex1 = self.translate_expression(children.pop().unwrap())?;
                Ok(match (ex1, ex2) {
                    (ExprOrNum::Num(r1), ExprOrNum::Num(r2)) => {
                        ExprOrNum::Num(if r1 != 0.0 || r2 != 0.0 { 1.0 } else { 0.0 })
                    }
                    (ex1, ex2) => ExprOrNum::Expr(Box::new(Expression::Or(ex1, ex2))),
                })
            }
            // expression: expression AND expression
            ASTNode::Inner(15, mut children) => {
                let ex2 = self.translate_expression(children.pop().unwrap())?;
                children.pop();
                let ex1 = self.translate_expression(children.pop().unwrap())?;
                Ok(match (ex1, ex2) {
                    (ExprOrNum::Num(r1), ExprOrNum::Num(r2)) => {
                        ExprOrNum::Num(if r1 != 0.0 && r2 != 0.0 { 1.0 } else { 0.0 })
                    }
                    (ex1, ex2) => ExprOrNum::Expr(Box::new(Expression::And(ex1, ex2))),
                })
            }
            // expression: expression '?' expression ':' expression
            ASTNode::Inner(16, mut children) => {
                let ex2 = self.translate_expression(children.pop().unwrap())?;
                children.pop();
                let ex1 = self.translate_expression(children.pop().unwrap())?;
                children.pop();
                let cond = self.translate_expression(children.pop().unwrap())?;
                Ok(match cond {
                    ExprOrNum::Expr(ex) => {
                        ExprOrNum::Expr(Box::new(Expression::Condition(ex, ex1, ex2)))
                    }
                    ExprOrNum::Num(r) => {
                        if r != 0.0 {
                            ex1
                        } else {
                            ex2
                        }
                    }
                })
            }
            // expression: IDENT '(' parameter_list ')'
            ASTNode::Inner(17, mut children) => {
                children.pop();
                let params = self.translate_parameter_list(children.pop().unwrap())?;
                children.pop();
                let ident = children.pop().unwrap().assume_leaf().assume_ident();
                if ident == self.cur_ident {
                    if params.len() != self.cur_variables.len() {
                        return Err(InputError::InconsistentVariablesCount { ident });
                    }
                    Ok(ExprOrNum::Expr(Box::new(Expression::Invoke(None, params))))
                } else {
                    match self.functions.get(&ident) {
                        Some(f) => {
                            if params.len() != f.incount {
                                return Err(InputError::InconsistentVariablesCount { ident });
                            }
                            let mut nums = vec![];
                            for param in params.iter() {
                                match param {
                                    &ExprOrNum::Expr(_) => break,
                                    &ExprOrNum::Num(r) => nums.push(r),
                                }
                            }
                            Ok(if params.len() == nums.len() {
                                ExprOrNum::Num(f.invoke(&nums))
                            } else {
                                ExprOrNum::Expr(Box::new(Expression::Invoke(
                                    Some(f.clone()),
                                    params,
                                )))
                            })
                        }
                        None => Err(InputError::UndefinedIdentifier { ident }),
                    }
                }
            }
            // expression: IDENT
            ASTNode::Inner(18, mut children) => {
                let ident = children.pop().unwrap().assume_leaf().assume_ident();
                match self.cur_variables.iter().position(|v| *v == ident) {
                    Some(i) => Ok(ExprOrNum::Expr(Box::new(Expression::Variable(i)))),
                    None => match self.values.get(&ident) {
                        Some((_, val)) => Ok(ExprOrNum::Num(*val)),
                        None => Err(InputError::UndefinedIdentifier { ident }),
                    },
                }
            }
            // expression: NUM
            ASTNode::Inner(19, mut children) => {
                let num = children.pop().unwrap().assume_leaf().assume_num();
                Ok(ExprOrNum::Num(num))
            }
            _ => unreachable!(),
        }
    }

    fn translate_variable_list(&self, ast: ASTNode) -> Result<Vec<Ident>, InputError> {
        let mut variables = vec![];
        let mut cur = ast;
        loop {
            match cur {
                // variable_list: variable_list ',' IDENT
                ASTNode::Inner(5, mut children) => {
                    let ident = children.pop().unwrap().assume_leaf().assume_ident();
                    if self.is_builtin_value(&ident) {
                        return Err(InputError::BuiltinIdentifier { ident });
                    }
                    variables.push(ident);
                    children.pop();
                    cur = children.pop().unwrap();
                }
                // variable_list: IDENT
                ASTNode::Inner(6, mut children) => {
                    let ident = children.pop().unwrap().assume_leaf().assume_ident();
                    if self.is_builtin_value(&ident) {
                        return Err(InputError::BuiltinIdentifier { ident });
                    }
                    variables.push(ident);
                    return Ok(variables);
                }
                _ => unreachable!(),
            }
        }
    }

    fn translate_parameter_list(&self, ast: ASTNode) -> Result<Vec<ExprOrNum>, InputError> {
        let mut params = vec![];
        let mut cur = ast;
        loop {
            match cur {
                // parameter_list: parameter_list ',' expression
                ASTNode::Inner(20, mut children) => {
                    let expr = self.translate_expression(children.pop().unwrap())?;
                    params.push(expr);
                    children.pop();
                    cur = children.pop().unwrap();
                }
                // parameter_list: expression
                ASTNode::Inner(21, mut children) => {
                    let expr = self.translate_expression(children.pop().unwrap())?;
                    params.push(expr);
                    return Ok(params);
                }
                _ => unreachable!(),
            }
        }
    }

    fn is_builtin_value(&self, ident: &Ident) -> bool {
        match self.values.get(ident) {
            Some((builtin, _)) => *builtin,
            None => false,
        }
    }

    fn is_builtin(&self, ident: &Ident) -> bool {
        self.is_builtin_value(ident)
            || match self.functions.get(ident) {
                Some(f) => match f.fimpl {
                    FunctionImpl::Lib(_) => true,
                    FunctionImpl::User(_) => false,
                },
                None => false,
            }
    }
}

impl Function {
    fn builtin(incount: usize, f: fn(&[Real]) -> Real) -> Arc<Self> {
        Arc::new(Function {
            incount,
            fimpl: FunctionImpl::Lib(f),
        })
    }

    fn invoke(&self, args: &[Real]) -> Real {
        match &self.fimpl {
            FunctionImpl::Lib(f) => f(args),
            FunctionImpl::User(expr) => self.calc_expr_or_num(expr, args),
        }
    }

    fn calc_expr_or_num(&self, expr: &ExprOrNum, args: &[Real]) -> Real {
        match expr {
            ExprOrNum::Expr(expr) => self.calc_expr(expr, args),
            ExprOrNum::Num(r) => *r,
        }
    }

    fn calc_expr(&self, expr: &Expression, args: &[Real]) -> Real {
        match expr {
            Expression::Not(expr) => match self.calc_expr(expr, args) == 0.0 {
                true => 1.0,
                false => 0.0,
            },
            Expression::Neg(expr) => -self.calc_expr(expr, args),
            Expression::Exp(ex1, ex2) => self
                .calc_expr_or_num(ex1, args)
                .powf(self.calc_expr_or_num(ex2, args)),
            Expression::Mul(ex1, ex2) => {
                self.calc_expr_or_num(ex1, args) * self.calc_expr_or_num(ex2, args)
            }
            Expression::Div(ex1, ex2) => {
                self.calc_expr_or_num(ex1, args) / self.calc_expr_or_num(ex2, args)
            }
            Expression::Add(ex1, ex2) => {
                self.calc_expr_or_num(ex1, args) + self.calc_expr_or_num(ex2, args)
            }
            Expression::Sub(ex1, ex2) => {
                self.calc_expr_or_num(ex1, args) - self.calc_expr_or_num(ex2, args)
            }
            Expression::Compare(cmp, ex1, ex2) => cmp.on(
                self.calc_expr_or_num(ex1, args),
                self.calc_expr_or_num(ex2, args),
            ),
            Expression::Or(ex1, ex2) => match self.calc_expr_or_num(ex1, args) != 0.0
                || self.calc_expr_or_num(ex2, args) != 0.0
            {
                true => 1.0,
                false => 0.0,
            },
            Expression::And(ex1, ex2) => match self.calc_expr_or_num(ex1, args) != 0.0
                && self.calc_expr_or_num(ex2, args) != 0.0
            {
                true => 1.0,
                false => 0.0,
            },
            Expression::Condition(expr, ex1, ex2) => match self.calc_expr(expr, args) != 0.0 {
                true => self.calc_expr_or_num(ex1, args),
                false => self.calc_expr_or_num(ex2, args),
            },
            Expression::Invoke(f, expr) => {
                let args = expr
                    .iter()
                    .map(|e| self.calc_expr_or_num(e, args))
                    .collect::<Vec<_>>();
                match f {
                    Some(f) => f.invoke(args.as_slice()),
                    None => self.invoke(args.as_slice()),
                }
            }
            Expression::Variable(i) => args[*i],
        }
    }
}
