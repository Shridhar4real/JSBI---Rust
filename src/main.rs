#![deny(unsafe_code)]

use std::env;
use std::io::{self, BufRead};
use jsbi::{JSBI, JSBIError};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "--ipc" {
        run_ipc_loop();
        return;
    }

    if args.len() >= 3 && args[1] == "eval" {
        let op = &args[2];
        let a_str = args.get(3).map(|s| s.as_str()).unwrap_or("0");
        let b_str = args.get(4).map(|s| s.as_str()).unwrap_or("0");

        match evaluate_op(op, a_str, b_str, None, None) {
            Ok(res) => println!("{}", res),
            Err(err) => {
                eprintln!("Error: {}", err);
                std::process::exit(1);
            }
        }
        return;
    }

    println!("JSBI Rust Port CLI (Port Mortem 2026 - Track F)");
    println!("Usage:");
    println!("  jsbi-cli eval <op> <a> [b]");
    println!("  jsbi-cli --ipc");
}

fn run_ipc_loop() {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut line = String::new();

    while handle.read_line(&mut line).unwrap_or(0) > 0 {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            line.clear();
            continue;
        }

        // Minimal std-only line protocol
        let parts: Vec<&str> = trimmed.split('|').collect();
        if !parts.is_empty() {
            let op = parts[0];
            let a_str = parts.get(1).copied().unwrap_or("0");
            let b_str = parts.get(2).copied().unwrap_or("0");

            let res = evaluate_op(op, a_str, b_str, None, None);
            match res {
                Ok(val) => println!("OK:{}", val),
                Err(err) => println!("ERR:{}", err),
            }
        }
        line.clear();
    }
}

fn evaluate_op(op: &str, a_str: &str, b_str: &str, _n_opt: Option<u32>, _radix_opt: Option<u32>) -> Result<String, JSBIError> {
    let a = JSBI::from_str(a_str)?;
    
    match op {
        "BigInt" | "parse" => Ok(a.to_string(10)),
        "toString" => {
            let radix = b_str.parse::<u32>().unwrap_or(10);
            Ok(a.to_string(radix))
        },
        "toNumber" => Ok(a.to_number().to_string()),
        "unaryMinus" | "NEG" | "neg" => Ok(JSBI::unary_minus(&a).to_string(10)),
        "bitwiseNot" | "NOT" | "not" => Ok(JSBI::bitwise_not(&a).to_string(10)),
        "add" | "ADD" => {
            let b = JSBI::from_str(b_str)?;
            Ok(JSBI::add(&a, &b).to_string(10))
        }
        "subtract" | "SUB" | "sub" => {
            let b = JSBI::from_str(b_str)?;
            Ok(JSBI::subtract(&a, &b).to_string(10))
        }
        "multiply" | "MUL" | "mul" => {
            let b = JSBI::from_str(b_str)?;
            Ok(JSBI::multiply(&a, &b).to_string(10))
        }
        "divide" | "DIV" | "div" => {
            let b = JSBI::from_str(b_str)?;
            Ok(JSBI::divide(&a, &b)?.to_string(10))
        }
        "remainder" | "MOD" | "mod" => {
            let b = JSBI::from_str(b_str)?;
            Ok(JSBI::remainder(&a, &b)?.to_string(10))
        }
        "exponentiate" | "EXP" | "exp" => {
            let b = JSBI::from_str(b_str)?;
            Ok(JSBI::exponentiate(&a, &b)?.to_string(10))
        }
        "bitwiseAnd" | "AND" | "and" => {
            let b = JSBI::from_str(b_str)?;
            Ok(JSBI::bitwise_and(&a, &b).to_string(10))
        }
        "bitwiseOr" | "OR" | "or" => {
            let b = JSBI::from_str(b_str)?;
            Ok(JSBI::bitwise_or(&a, &b).to_string(10))
        }
        "bitwiseXor" | "XOR" | "xor" => {
            let b = JSBI::from_str(b_str)?;
            Ok(JSBI::bitwise_xor(&a, &b).to_string(10))
        }
        "leftShift" | "SHL" | "shl" => {
            let b = JSBI::from_str(b_str)?;
            Ok(JSBI::left_shift(&a, &b).to_string(10))
        }
        "signedRightShift" | "SAR" | "sar" => {
            let b = JSBI::from_str(b_str)?;
            Ok(JSBI::signed_right_shift(&a, &b).to_string(10))
        }
        "equal" | "EQ" => {
            let b = JSBI::from_str(b_str)?;
            Ok(JSBI::equal(&a, &b).to_string())
        }
        "lessThan" | "LT" => {
            let b = JSBI::from_str(b_str)?;
            Ok(JSBI::less_than(&a, &b).to_string())
        }
        "greaterThan" | "GT" => {
            let b = JSBI::from_str(b_str)?;
            Ok(JSBI::greater_than(&a, &b).to_string())
        }
        "asIntN" => {
            let n = b_str.parse::<f64>().map(|f| f as u32).unwrap_or_else(|_| _n_opt.unwrap_or(64));
            Ok(JSBI::as_int_n(n, &a).to_string(10))
        }
        "asUintN" => {
            let n = b_str.parse::<f64>().map(|f| f as u32).unwrap_or_else(|_| _n_opt.unwrap_or(64));
            Ok(JSBI::as_uint_n(n, &a).to_string(10))
        }
        _ => Err(JSBIError::GenericError(format!("Unknown operation {}", op))),
    }
}
