use crate::ast::Expr;
use crate::err::{IntErr, Interrupt};
use crate::num::{Base, FormattingStyle, Number};
use crate::{parser::ParseOptions, scope::Scope};
use std::fmt;

#[derive(Debug, Clone)]
pub enum Value {
    Num(Number),
    // built-in function
    BuiltInFunction(BuiltInFunction),
    Format(FormattingStyle),
    Dp,
    Base(Base),
    // user-defined function with a named parameter
    Fn(String, Expr, Scope),
    Version,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum BuiltInFunction {
    Approximately,
    Abs,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Sinh,
    Cosh,
    Tanh,
    Asinh,
    Acosh,
    Atanh,
    Ln,
    Log2,
    Log10,
    Base,
}

impl fmt::Display for BuiltInFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            Self::Approximately => "approximately",
            Self::Abs => "abs",
            Self::Sin => "sin",
            Self::Cos => "cos",
            Self::Tan => "tan",
            Self::Asin => "asin",
            Self::Acos => "acos",
            Self::Atan => "atan",
            Self::Sinh => "sinh",
            Self::Cosh => "cosh",
            Self::Tanh => "tanh",
            Self::Asinh => "asinh",
            Self::Acosh => "acosh",
            Self::Atanh => "atanh",
            Self::Ln => "ln",
            Self::Log2 => "log2",
            Self::Log10 => "log10",
            Self::Base => "base",
        };
        write!(f, "{}", name)
    }
}

impl Value {
    pub fn expect_num<I: Interrupt>(self) -> Result<Number, IntErr<String, I>> {
        match self {
            Self::Num(bigrat) => Ok(bigrat),
            _ => Err("Expected a number".to_string())?,
        }
    }

    pub fn apply<I: Interrupt>(
        self,
        other: Expr,
        allow_multiplication: bool,
        force_multiplication: bool,
        scope: &mut Scope,
        options: ParseOptions,
        int: &I,
    ) -> Result<Self, IntErr<String, I>> {
        Ok(Self::Num(match self {
            Self::Num(n) => {
                let other = crate::ast::evaluate(other, scope, options, int)?;
                if let Self::Dp = other {
                    let num = Self::Num(n).expect_num()?.try_as_usize(int)?;
                    return Ok(Self::Format(FormattingStyle::ApproxFloat(num)));
                }
                if allow_multiplication {
                    n.clone().mul(other.expect_num()?, int)?
                } else {
                    let self_ = Self::Num(n);
                    return Err(format!(
                        "{} is not a function",
                        crate::num::to_string(|f| self_.format(f, int))?.0
                    ))?;
                }
            }
            Self::BuiltInFunction(name) => {
                let other = crate::ast::evaluate(other, scope, options, int)?;
                if force_multiplication {
                    return Err(format!(
                        "Cannot apply function '{}' in this context",
                        crate::num::to_string(|f| self.format(f, int))?.0
                    ))?;
                }
                match name {
                    BuiltInFunction::Approximately => other.expect_num()?.make_approximate(),
                    BuiltInFunction::Abs => other.expect_num()?.abs(int)?,
                    BuiltInFunction::Sin => other.expect_num()?.sin(scope, int)?,
                    BuiltInFunction::Cos => other.expect_num()?.cos(scope, int)?,
                    BuiltInFunction::Tan => other.expect_num()?.tan(scope, int)?,
                    BuiltInFunction::Asin => other.expect_num()?.asin(int)?,
                    BuiltInFunction::Acos => other.expect_num()?.acos(int)?,
                    BuiltInFunction::Atan => other.expect_num()?.atan(int)?,
                    BuiltInFunction::Sinh => other.expect_num()?.sinh(int)?,
                    BuiltInFunction::Cosh => other.expect_num()?.cosh(int)?,
                    BuiltInFunction::Tanh => other.expect_num()?.tanh(int)?,
                    BuiltInFunction::Asinh => other.expect_num()?.asinh(int)?,
                    BuiltInFunction::Acosh => other.expect_num()?.acosh(int)?,
                    BuiltInFunction::Atanh => other.expect_num()?.atanh(int)?,
                    BuiltInFunction::Ln => other.expect_num()?.ln(int)?,
                    BuiltInFunction::Log2 => other.expect_num()?.log2(int)?,
                    BuiltInFunction::Log10 => other.expect_num()?.log10(int)?,
                    BuiltInFunction::Base => {
                        use std::convert::TryInto;
                        let n: u8 = other
                            .expect_num()?
                            .try_as_usize(int)?
                            .try_into()
                            .map_err(|_| "Unable to convert number to a valid base".to_string())?;
                        return Ok(Self::Base(
                            Base::from_plain_base(n).map_err(|e| e.to_string())?,
                        ));
                    }
                }
            }
            Self::Fn(param, expr, custom_scope) => {
                let mut new_scope = custom_scope.clone().create_nested_scope();
                new_scope.insert_variable(param.clone(), other, scope.clone(), options);
                return Ok(crate::ast::evaluate(
                    expr.clone(),
                    &mut new_scope,
                    options,
                    int,
                )?);
            }
            _ => {
                return Err(format!(
                    "'{}' is not a function or a number",
                    crate::num::to_string(|f| self.format(f, int))?.0
                ))?;
            }
        }))
    }

    pub fn format<I: Interrupt>(
        &self,
        f: &mut fmt::Formatter,
        int: &I,
    ) -> Result<(), IntErr<fmt::Error, I>> {
        match self {
            Self::Num(n) => write!(f, "{}", crate::num::to_string(|f| n.format(f, int))?.0)?,
            Self::BuiltInFunction(name) => write!(f, "{}", name)?,
            Self::Format(fmt) => write!(f, "{}", fmt)?,
            Self::Dp => write!(f, "dp")?,
            Self::Base(b) => write!(f, "base {}", b.base_as_u8())?,
            Self::Fn(name, expr, _scope) => {
                let expr_str = crate::num::to_string(|f| expr.format(f, int))?.0;
                if name.contains('.') {
                    write!(f, "{}:{}", name, expr_str)?
                } else {
                    write!(f, "\\{}.{}", name, expr_str)?
                }
            }
            Self::Version => write!(f, "{}", crate::get_version())?,
        }
        Ok(())
    }
}
