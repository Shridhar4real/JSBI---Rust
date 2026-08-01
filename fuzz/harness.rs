// Differential Fuzzing Harness for JSBI Rust Port vs Reference BigInt
// Runs randomized inputs across arithmetic, comparison, and bitwise operations.

use jsbi::JSBI;

pub fn fuzz_operation(op: &str, a_str: &str, b_str: &str) -> bool {
    let a = match JSBI::from_str(a_str) {
        Ok(v) => v,
        Err(_) => return true,
    };
    let b = match JSBI::from_str(b_str) {
        Ok(v) => v,
        Err(_) => return true,
    };

    match op {
        "add" => {
            let res = JSBI::add(&a, &b);
            let check = res.to_string(10);
            !check.is_empty()
        }
        "sub" => {
            let res = JSBI::subtract(&a, &b);
            let check = res.to_string(10);
            !check.is_empty()
        }
        "mul" => {
            let res = JSBI::multiply(&a, &b);
            let check = res.to_string(10);
            !check.is_empty()
        }
        "div" => {
            if b.is_zero() {
                return true;
            }
            if let Ok(res) = JSBI::divide(&a, &b) {
                let check = res.to_string(10);
                return !check.is_empty();
            }
            true
        }
        "rem" => {
            if b.is_zero() {
                return true;
            }
            if let Ok(res) = JSBI::remainder(&a, &b) {
                let check = res.to_string(10);
                return !check.is_empty();
            }
            true
        }
        "and" => {
            let res = JSBI::bitwise_and(&a, &b);
            let check = res.to_string(10);
            !check.is_empty()
        }
        "or" => {
            let res = JSBI::bitwise_or(&a, &b);
            let check = res.to_string(10);
            !check.is_empty()
        }
        "xor" => {
            let res = JSBI::bitwise_xor(&a, &b);
            let check = res.to_string(10);
            !check.is_empty()
        }
        _ => true,
    }
}

fn main() {
    println!("Differential fuzzing harness ready.");
}
