/*
 * Copyright 2022-2023 Amazon.com, Inc. or its affiliates. All Rights Reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! This module contains the Cedar 'u256' extension.

use regex::Regex;

use crate::ast::{
    CallStyle, Extension, ExtensionFunction, ExtensionOutputValue, ExtensionValue,
    ExtensionValueWithArgs, Name, Value,
};
use crate::entities::SchemaType;
use crate::evaluator;
use std::sync::Arc;
use thiserror::Error;

use ethers::prelude::U256;


/// UINT256 value, represented internally as an integer.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct UINT256 {
    value: U256,
}

// PANIC SAFETY All `Name`s in here are valid `Name`s
#[allow(clippy::expect_used)]
mod names {
    use super::{Name, EXTENSION_NAME};
    // PANIC SAFETY all of the names here are valid names
    lazy_static::lazy_static! {
        // pub static ref DECIMAL_FROM_STR_NAME : Name = Name::parse_unqualified_name(EXTENSION_NAME).expect("should be a valid identifier");
        pub static ref UINT256_FROM_STR_NAME : Name = Name::parse_unqualified_name(EXTENSION_NAME).expect("should be a valid identifier");
        pub static ref LESS_THAN : Name = Name::parse_unqualified_name("u256LessThan").expect("should be a valid identifier");
        pub static ref LESS_THAN_OR_EQUAL : Name = Name::parse_unqualified_name("u256LessThanOrEqual").expect("should be a valid identifier");
        pub static ref GREATER_THAN : Name = Name::parse_unqualified_name("u256GreaterThan").expect("should be a valid identifier");
        pub static ref GREATER_THAN_OR_EQUAL : Name = Name::parse_unqualified_name("u256GreaterThanOrEqual").expect("should be a valid identifier");
    }
}

/// Potential errors when working with u256 values. Note that these are
/// converted to evaluator::Err::ExtensionErr (which takes a string argument)
/// before being reported to users.
#[derive(Debug, Error)]
enum Error {
    /// Error parsing the input string as a u256 value
    #[error("input string is not a well-formed u256 value: {0}")]
    FailedParse(String),

    /// Overflow occurred when converting to a u256 value
    #[error("overflow when converting to u256")]
    Overflow,
}


impl UINT256 {
    /// The Cedar typename of u256 values
    fn typename() -> Name {
        names::UINT256_FROM_STR_NAME.clone()
    }

    /// Convert a string into a `UINT256` value.
    ///
    /// Matches against the regular expression `-?[0-9]+.[0-9]+`, which requires
    /// only int digits
    ///
    fn from_str(str: impl AsRef<str>) -> Result<Self, Error> {
        // check that the string matches the regex
        // PANIC SAFETY: This regex does parse
        #[allow(clippy::unwrap_used)]
        let re = Regex::new(r#"^[0-9]\d*$"#).unwrap();
        if !re.is_match(str.as_ref()) {
            return Err(Error::FailedParse(str.as_ref().to_owned()));
        }

        let l = U256::from_dec_str(str.as_ref()).map_err(|_| Error::Overflow)?;

         Ok(Self { value: l })
        // l.map(|value| Self { value })
        // .ok_or(Error::Overflow)
    }
}

impl std::fmt::Display for UINT256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.value
        )
    }
}

impl ExtensionValue for UINT256 {
    fn typename(&self) -> Name {
        Self::typename()
    }
}

const EXTENSION_NAME: &str = "u256";

fn extension_err(msg: impl Into<String>) -> evaluator::EvaluationError {
    evaluator::EvaluationError::ExtensionError {
        extension_name: names::UINT256_FROM_STR_NAME.clone(),
        msg: msg.into(),
    }
}

/// Cedar function that constructs a `u256` Cedar type from a
/// Cedar string
fn uint256_from_str(arg: Value) -> evaluator::Result<ExtensionOutputValue> {
    let str = arg.get_as_string()?;
    let u256 = UINT256::from_str(str.as_str()).map_err(|e| extension_err(e.to_string()))?;
    let function_name = names::UINT256_FROM_STR_NAME.clone();
    let e = ExtensionValueWithArgs::new(Arc::new(u256), vec![arg.into()], function_name);
    Ok(Value::ExtensionValue(Arc::new(e)).into())
}


/// Cedar function that tests whether the first `u256` Cedar type is
/// less than the second `u256` Cedar type, returning a Cedar bool
fn uint256_lt(left: Value, right: Value) -> evaluator::Result<ExtensionOutputValue> {
    Ok(Value::Lit((left.lt(&right)).into()).into())
}

/// Cedar function that tests whether the first `u256` Cedar type is
/// less than or equal to the second `u256` Cedar type, returning a Cedar bool
fn uint256_le(left: Value, right: Value) -> evaluator::Result<ExtensionOutputValue> {
    Ok(Value::Lit((left.le(&right)).into()).into())
}

/// Cedar function that tests whether the first `u256` Cedar type is
/// greater than the second `u256` Cedar type, returning a Cedar bool
fn uint256_gt(left: Value, right: Value) -> evaluator::Result<ExtensionOutputValue> {
    Ok(Value::Lit((left.gt(&right)).into()).into())
}

/// Cedar function that tests whether the first `u256` Cedar type is
/// greater than or equal to the second `u256` Cedar type, returning a Cedar bool
fn uint256_ge(left: Value, right: Value) -> evaluator::Result<ExtensionOutputValue> {
    Ok(Value::Lit((left.ge(&right)).into()).into())
}

/// Construct the extension
pub fn extension() -> Extension {
    let uint256_type = SchemaType::Extension {
        name: UINT256::typename(),
    };
    Extension::new(
        names::UINT256_FROM_STR_NAME.clone(),
        vec![
            ExtensionFunction::unary(
                names::UINT256_FROM_STR_NAME.clone(),
                CallStyle::FunctionStyle,
                Box::new(uint256_from_str),
                uint256_type.clone(),
                Some(SchemaType::String),
            ),
            ExtensionFunction::binary(
                names::LESS_THAN.clone(),
                CallStyle::MethodStyle,
                Box::new(uint256_lt),
                SchemaType::Bool,
                (Some(uint256_type.clone()), Some(uint256_type.clone())),
            ),
            ExtensionFunction::binary(
                names::LESS_THAN_OR_EQUAL.clone(),
                CallStyle::MethodStyle,
                Box::new(uint256_le),
                SchemaType::Bool,
                (Some(uint256_type.clone()), Some(uint256_type.clone())),
            ),
            ExtensionFunction::binary(
                names::GREATER_THAN.clone(),
                CallStyle::MethodStyle,
                Box::new(uint256_gt),
                SchemaType::Bool,
                (Some(uint256_type.clone()), Some(uint256_type.clone())),
            ),
            ExtensionFunction::binary(
                names::GREATER_THAN_OR_EQUAL.clone(),
                CallStyle::MethodStyle,
                Box::new(uint256_ge),
                SchemaType::Bool,
                (Some(uint256_type.clone()), Some(uint256_type)),
            ),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Type, Value};
    use crate::evaluator::test::{basic_entities, basic_request};
    use crate::evaluator::Evaluator;
    use crate::extensions::Extensions;
    use crate::parser::parse_expr;

    /// Asserts that a `Result` is an `Err::ExtensionErr` with our extension name
    fn assert_uint256_err<T>(res: evaluator::Result<T>) {
        match res {
            Err(evaluator::EvaluationError::ExtensionError {
                extension_name,
                msg,
            }) => {
                println!("{msg}");
                assert_eq!(
                    extension_name,
                    Name::parse_unqualified_name("u256").expect("should be a valid identifier")
                )
            }
            Err(e) => panic!("Expected an u256 ExtensionErr, got {:?}", e),
            Ok(_) => panic!("Expected an u256 ExtensionErr, got Ok"),
        }
    }

    /// Asserts that a `Result` is a u256 value
    fn assert_uint256_valid(res: evaluator::Result<Value>) {
        match res {
            Ok(Value::ExtensionValue(ev)) => {
                assert_eq!(ev.typename(), UINT256::typename())
            }
            Ok(v) => panic!("Expected u256 ExtensionValue, got {:?}", v),
            Err(e) => panic!("Expected Ok, got Err: {:?}", e),
        }
    }

    /// this test just ensures that the right functions are marked constructors
    #[test]
    fn constructors() {
        let ext = extension();
        assert!(ext
            .get_func(
                &Name::parse_unqualified_name("u256").expect("should be a valid identifier")
            )
            .expect("function should exist")
            .is_constructor());
        assert!(!ext
            .get_func(
                &Name::parse_unqualified_name("u256LessThan").expect("should be a valid identifier")
            )
            .expect("function should exist")
            .is_constructor());
        assert!(!ext
            .get_func(
                &Name::parse_unqualified_name("u256LessThanOrEqual")
                    .expect("should be a valid identifier")
            )
            .expect("function should exist")
            .is_constructor());
        assert!(!ext
            .get_func(
                &Name::parse_unqualified_name("u256GreaterThan").expect("should be a valid identifier")
            )
            .expect("function should exist")
            .is_constructor());
        assert!(!ext
            .get_func(
                &Name::parse_unqualified_name("u256GreaterThanOrEqual")
                    .expect("should be a valid identifier")
            )
            .expect("function should exist")
            .is_constructor(),);
    }

    #[test]
    fn uint256_creation() {
        let ext_array = [extension()];
        let exts = Extensions::specific_extensions(&ext_array);
        let request = basic_request();
        let entities = basic_entities();
        let eval = Evaluator::new(&request, &entities, &exts).unwrap();

        // valid u256 strings
        assert_uint256_valid(
            eval.interpret_inline_policy(&parse_expr(r#"u256("115792089237316195423570985008687907853269984665640564039457584007913129639935")"#).expect("parsing error")),
        );
        assert_uint256_valid(
            eval.interpret_inline_policy(&parse_expr(r#"u256("0")"#).expect("parsing error")),
        );
        assert_uint256_valid(
            eval.interpret_inline_policy(
                &parse_expr(r#"u256("123456")"#).expect("parsing error"),
            ),
        );
        assert_uint256_valid(
            eval.interpret_inline_policy(
                &parse_expr(r#"u256("1234")"#).expect("parsing error"),
            ),
        );

        // invalid u256 strings
        assert_uint256_err(
            eval.interpret_inline_policy(&parse_expr(r#"u256("12.34")"#).expect("parsing error")),
        );
        assert_uint256_err(
            eval.interpret_inline_policy(&parse_expr(r#"u256("1.0.")"#).expect("parsing error")),
        );
        assert_uint256_err(
            eval.interpret_inline_policy(&parse_expr(r#"u256("1.")"#).expect("parsing error")),
        );
        assert_uint256_err(
            eval.interpret_inline_policy(&parse_expr(r#"u256(".1")"#).expect("parsing error")),
        );
        assert_uint256_err(
            eval.interpret_inline_policy(&parse_expr(r#"u256("1.a")"#).expect("parsing error")),
        );
        assert_uint256_err(
            eval.interpret_inline_policy(&parse_expr(r#"u256("-.")"#).expect("parsing error")),
        );


        // bad use of `u256` as method
        parse_expr(r#" "1.0".u256() "#).expect_err("should fail");
    }

    #[test]
    fn uint256_equality() {
        let ext_array = [extension()];
        let exts = Extensions::specific_extensions(&ext_array);
        let request = basic_request();
        let entities = basic_entities();
        let eval = Evaluator::new(&request, &entities, &exts).unwrap();

        let a = parse_expr(r#"u256("123")"#).expect("parsing error");
        let b = parse_expr(r#"u256("123")"#).expect("parsing error");
        let c = parse_expr(r#"u256("123")"#).expect("parsing error");
        let d = parse_expr(r#"u256("124")"#).expect("parsing error");
        let e = parse_expr(r#"u256("1")"#).expect("parsing error");
        let f = parse_expr(r#"u256("0")"#).expect("parsing error");
        let g = parse_expr(r#"u256("0")"#).expect("parsing error");

        // a, b, c are all equal
        assert_eq!(
            eval.interpret_inline_policy(&Expr::is_eq(a.clone(), a.clone())),
            Ok(Value::from(true))
        );
        assert_eq!(
            eval.interpret_inline_policy(&Expr::is_eq(a.clone(), b.clone())),
            Ok(Value::from(true))
        );
        assert_eq!(
            eval.interpret_inline_policy(&Expr::is_eq(b.clone(), c.clone())),
            Ok(Value::from(true))
        );
        assert_eq!(
            eval.interpret_inline_policy(&Expr::is_eq(c, a.clone())),
            Ok(Value::from(true))
        );

        // d, e are distinct
        assert_eq!(
            eval.interpret_inline_policy(&Expr::is_eq(b, d.clone())),
            Ok(Value::from(false))
        );
        assert_eq!(
            eval.interpret_inline_policy(&Expr::is_eq(a.clone(), e.clone())),
            Ok(Value::from(false))
        );
        assert_eq!(
            eval.interpret_inline_policy(&Expr::is_eq(d, e)),
            Ok(Value::from(false))
        );

        // f (0.0) and g (-0.0) are equal
        assert_eq!(
            eval.interpret_inline_policy(&Expr::is_eq(f, g)),
            Ok(Value::from(true))
        );

        // other types are not equal
        assert_eq!(
            eval.interpret_inline_policy(&Expr::is_eq(a.clone(), Expr::val("123.0"))),
            Ok(Value::from(false))
        );
        assert_eq!(
            eval.interpret_inline_policy(&Expr::is_eq(a, Expr::val(1))),
            Ok(Value::from(false))
        );
    }

    fn uint256_ops_helper(op: &str, tests: Vec<((Expr, Expr), bool)>) {
        let ext_array = [extension()];
        let exts = Extensions::specific_extensions(&ext_array);
        let request = basic_request();
        let entities = basic_entities();
        let eval = Evaluator::new(&request, &entities, &exts).unwrap();

        for ((l, r), res) in tests {
            assert_eq!(
                eval.interpret_inline_policy(&Expr::call_extension_fn(
                    Name::parse_unqualified_name(op).expect("should be a valid identifier"),
                    vec![l, r]
                )),
                Ok(Value::from(res))
            );
        }
    }

    #[test]
    fn uint256_ops() {
        let a = parse_expr(r#"u256("123")"#).expect("parsing error");
        let b = parse_expr(r#"u256("124")"#).expect("parsing error");

        // tests for u256LessThan
        let tests = vec![
            ((a.clone(), b.clone()), true),  // 123 < 124
            ((a.clone(), a.clone()), false), // 123 < 123
        ];
        uint256_ops_helper("u256LessThan", tests);

        // tests for u256LessThanOrEqual
        let tests = vec![
            ((a.clone(), b.clone()), true),  // 123 <= 124
            ((a.clone(), a.clone()), true),  // 123 <= 123
        ];
        uint256_ops_helper("u256LessThanOrEqual", tests);

        // tests for u256GreaterThan
        let tests = vec![
            ((a.clone(), b.clone()), false), // 123 > 124
            ((a.clone(), a.clone()), false), // 123 > 123
        ];
        uint256_ops_helper("u256GreaterThan", tests);

        // tests for u256GreaterThanOrEqual
        let tests = vec![
            ((a.clone(), b), false),        // 123 >= 124
            ((a.clone(), a.clone()), true), // 123 >= 123
        ];
        uint256_ops_helper("u256GreaterThanOrEqual", tests);

        // evaluation errors

        let ext_array = [extension()];
        let exts = Extensions::specific_extensions(&ext_array);
        let request = basic_request();
        let entities = basic_entities();
        let eval = Evaluator::new(&request, &entities, &exts).unwrap();

        assert_eq!(
            eval.interpret_inline_policy(
                &parse_expr(r#"u256("123") < u256("124")"#).expect("parsing error")
            ),
            Err(evaluator::EvaluationError::TypeError {
                expected: vec![Type::Long],
                actual: Type::Extension {
                    name: Name::parse_unqualified_name("u256")
                        .expect("should be a valid identifier")
                },
            })
        );
    }

    fn check_round_trip(s: &str) {
        let d = UINT256::from_str(s).expect("should be a valid u256");
        assert_eq!(s, d.to_string());
    }

    #[test]
    fn uint256_display() {
        // these strings will display the same after parsing
        check_round_trip("1230");
        check_round_trip("12300");
        check_round_trip("1234560");
    }
}
