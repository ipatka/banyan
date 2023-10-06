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

use crate::extension_schema::{ArgumentCheckFn, ExtensionFunctionType, ExtensionSchema};
use crate::types::{self, Type};
use cedar_policy_core::ast::{Expr, ExprKind, Literal, RestrictedExpr};
use cedar_policy_core::evaluator::RestrictedEvaluator;
use cedar_policy_core::extensions::{u256, Extensions};
use std::str::FromStr;

/// If any of the panics in this file are triggered, that means that this file has become
/// out-of-date with the u256 extension definition in CedarCore.

fn get_argument_types(fname: &str, u256_ty: &Type) -> Vec<types::Type> {
    match fname {
        "u256" => vec![Type::primitive_string()],
        "u256LessThan" | "u256LessThanOrEqual" | "u256GreaterThan" | "u256GreaterThanOrEqual" => {
            vec![u256_ty.clone(), u256_ty.clone()]
        }
        _ => panic!("unexpected u256 extension function name: {fname}"),
    }
}

fn get_return_type(fname: &str, u256_ty: &Type) -> Type {
    match fname {
        "u256" => u256_ty.clone(),
        "u256LessThan" | "u256LessThanOrEqual" | "u256GreaterThan" | "u256GreaterThanOrEqual" => {
            Type::primitive_boolean()
        }
        _ => panic!("unexpected u256 extension function name: {fname}"),
    }
}

fn get_argument_check(fname: &str) -> Option<ArgumentCheckFn> {
    match fname {
        "u256" => Some(Box::new(validate_u256_string)),
        "u256LessThan" | "u256LessThanOrEqual" | "u256GreaterThan" | "u256GreaterThanOrEqual" => None,
        _ => panic!("unexpected u256 extension function name: {fname}"),
    }
}

/// Construct the extension schema
pub fn extension_schema() -> ExtensionSchema {
    let u256_ext = u256::extension();
    let u256_ty = Type::extension(u256_ext.name().clone());

    let fun_tys: Vec<ExtensionFunctionType> = u256_ext
        .funcs()
        .map(|f| {
            let fname = f.name();
            let fstring = fname.to_string();
            let return_type = get_return_type(&fstring, &u256_ty);
            debug_assert!(f
                .return_type()
                .map(|ty| return_type.is_consistent_with(ty))
                .unwrap_or_else(|| return_type == Type::Never));
            ExtensionFunctionType::new(
                fname.clone(),
                get_argument_types(&fstring, &u256_ty),
                return_type,
                get_argument_check(&fstring),
            )
        })
        .collect();
    ExtensionSchema::new(u256_ext.name().clone(), fun_tys)
}

/// Extra validation step for the `u256` function.
/// Note that `exprs` will have already been checked to contain the correct number of arguments.
fn validate_u256_string(exprs: &[Expr]) -> Result<(), String> {
    match exprs.get(0) {
        Some(arg) if matches!(arg.expr_kind(), ExprKind::Lit(Literal::String(_))) => {
            let exts = Extensions::all_available();
            let evaluator = RestrictedEvaluator::new(&exts);
            match RestrictedExpr::from_str(&format!("u256({arg})")) {
                Ok(expr) => match evaluator.interpret(expr.as_borrowed()) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(format!("Failed to parse as a u256 value: `{arg}`")),
                },
                Err(_) => Err(format!("Failed to parse as a u256 value: `{arg}`")),
            }
        }
        _ => Ok(()),
    }
}
