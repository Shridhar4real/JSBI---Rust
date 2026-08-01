use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Sub, Mul, Div, Rem, BitAnd, BitOr, BitXor, Shl, Shr, Neg, Not};
use crate::error::JSBIError;

pub const DIGIT_BITS: u32 = 30;
pub const DIGIT_MASK: u32 = (1u32 << DIGIT_BITS) - 1; // 0x3FFFFFFF
pub const DIGIT_BASE: u64 = 1u64 << DIGIT_BITS;       // 1073741824

#[derive(Clone, Eq)]
pub struct JSBI {
    pub sign: bool, // true if negative, false if positive or zero
    pub digits: Vec<u32>, // 30-bit digits, least significant first
}

impl JSBI {
    pub fn zero() -> Self {
        JSBI {
            sign: false,
            digits: Vec::new(),
        }
    }

    pub fn is_zero(&self) -> bool {
        self.digits.is_empty()
    }

    pub fn one_digit(val: u32, sign: bool) -> Self {
        let val = val & DIGIT_MASK;
        if val == 0 {
            JSBI::zero()
        } else {
            JSBI {
                sign,
                digits: vec![val],
            }
        }
    }

    pub fn trim(mut self) -> Self {
        while let Some(&last) = self.digits.last() {
            if last == 0 {
                self.digits.pop();
            } else {
                break;
            }
        }
        if self.digits.is_empty() {
            self.sign = false;
        }
        self
    }

    pub fn from_i64(n: i64) -> Self {
        if n == 0 {
            return JSBI::zero();
        }
        let sign = n < 0;
        let mut unsigned = if sign {
            (n as i128).abs() as u64
        } else {
            n as u64
        };
        let mut digits = Vec::new();
        while unsigned > 0 {
            digits.push((unsigned & (DIGIT_MASK as u64)) as u32);
            unsigned >>= DIGIT_BITS;
        }
        JSBI { sign, digits }.trim()
    }

    pub fn from_u64(n: u64) -> Self {
        if n == 0 {
            return JSBI::zero();
        }
        let mut unsigned = n;
        let mut digits = Vec::new();
        while unsigned > 0 {
            digits.push((unsigned & (DIGIT_MASK as u64)) as u32);
            unsigned >>= DIGIT_BITS;
        }
        JSBI { sign: false, digits }.trim()
    }

    pub fn from_f64(val: f64) -> Result<Self, JSBIError> {
        if val == 0.0 {
            return Ok(JSBI::zero());
        }
        if !val.is_finite() || val.floor() != val {
            return Err(JSBIError::RangeError(format!(
                "The number {} cannot be converted to BigInt because it is not an integer",
                val
            )));
        }
        let bits = val.to_bits();
        let sign = (bits >> 63) != 0;
        let raw_exp = ((bits >> 52) & 0x7FF) as i32;
        let mantissa = (bits & 0x000F_FFFF_FFFF_FFFF) | 0x0010_0000_0000_0000;
        let exp = raw_exp - 1023 - 52;

        let mantissa_jsbi = JSBI::from_u64(mantissa);
        let res = if exp >= 0 {
            JSBI::left_shift(&mantissa_jsbi, &JSBI::from_u64(exp as u64))
        } else {
            JSBI::signed_right_shift(&mantissa_jsbi, &JSBI::from_u64((-exp) as u64))
        };
        let mut final_res = res;
        final_res.sign = sign;
        Ok(final_res.trim())
    }

    pub fn from_str(input: &str) -> Result<Self, JSBIError> {
        let s = input.trim();
        if s.is_empty() {
            return Err(JSBIError::SyntaxError(format!("Cannot convert {} to a BigInt", input)));
        }

        let (sign, s_nosign) = if let Some(stripped) = s.strip_prefix('-') {
            (true, stripped)
        } else if let Some(stripped) = s.strip_prefix('+') {
            (false, stripped)
        } else {
            (false, s)
        };

        if s_nosign.is_empty() {
            return Err(JSBIError::SyntaxError(format!("Cannot convert {} to a BigInt", input)));
        }

        let (radix, body) = if let Some(stripped) = s_nosign.strip_prefix("0x").or_else(|| s_nosign.strip_prefix("0X")) {
            if sign {
                return Err(JSBIError::SyntaxError(format!("Cannot convert {} to a BigInt", input)));
            }
            (16, stripped)
        } else if let Some(stripped) = s_nosign.strip_prefix("0o").or_else(|| s_nosign.strip_prefix("0O")) {
            if sign {
                return Err(JSBIError::SyntaxError(format!("Cannot convert {} to a BigInt", input)));
            }
            (8, stripped)
        } else if let Some(stripped) = s_nosign.strip_prefix("0b").or_else(|| s_nosign.strip_prefix("0B")) {
            if sign {
                return Err(JSBIError::SyntaxError(format!("Cannot convert {} to a BigInt", input)));
            }
            (2, stripped)
        } else {
            (10, s_nosign)
        };

        if body.is_empty() {
            return Err(JSBIError::SyntaxError(format!("Cannot convert {} to a BigInt", input)));
        }

        let mut result = JSBI::zero();
        let radix_big = JSBI::from_u64(radix as u64);

        for ch in body.chars() {
            let digit = match ch.to_digit(radix) {
                Some(d) => d,
                None => return Err(JSBIError::SyntaxError(format!("Cannot convert {} to a BigInt", input))),
            };
            result = JSBI::add(&JSBI::multiply(&result, &radix_big), &JSBI::from_u64(digit as u64));
        }

        if result.is_zero() {
            Ok(JSBI::zero())
        } else {
            result.sign = sign;
            Ok(result)
        }
    }

    pub fn to_string(&self, radix: u32) -> String {
        if !(2..=36).contains(&radix) {
            panic!("toString() radix argument must be between 2 and 36");
        }
        if self.is_zero() {
            return "0".to_string();
        }

        let mut current = JSBI { sign: false, digits: self.digits.clone() };
        let radix_big = JSBI::from_u64(radix as u64);
        let mut chars = Vec::new();

        let charset = b"0123456789abcdefghijklmnopqrstuvwxyz";

        while !current.is_zero() {
            let (q, r) = JSBI::abs_div_rem(&current, &radix_big);
            let rem_digit = if r.digits.is_empty() { 0 } else { r.digits[0] };
            chars.push(charset[rem_digit as usize] as char);
            current = q;
        }

        if self.sign {
            chars.push('-');
        }

        chars.into_iter().rev().collect()
    }

    pub fn to_number(&self) -> f64 {
        if self.is_zero() {
            return 0.0;
        }
        let mut res = 0.0;
        let mut mult = 1.0;
        for &d in &self.digits {
            res += (d as f64) * mult;
            mult *= DIGIT_BASE as f64;
        }
        if self.sign { -res } else { res }
    }

    // --- Absolute Value Helper Methods ---

    fn abs_cmp(x: &JSBI, y: &JSBI) -> Ordering {
        if x.digits.len() != y.digits.len() {
            return x.digits.len().cmp(&y.digits.len());
        }
        for i in (0..x.digits.len()).rev() {
            if x.digits[i] != y.digits[i] {
                return x.digits[i].cmp(&y.digits[i]);
            }
        }
        Ordering::Equal
    }

    fn abs_add(x: &JSBI, y: &JSBI) -> JSBI {
        let len = x.digits.len().max(y.digits.len());
        let mut digits = Vec::with_capacity(len + 1);
        let mut carry = 0u64;

        for i in 0..len {
            let dx = *x.digits.get(i).unwrap_or(&0) as u64;
            let dy = *y.digits.get(i).unwrap_or(&0) as u64;
            let sum = dx + dy + carry;
            digits.push((sum & (DIGIT_MASK as u64)) as u32);
            carry = sum >> DIGIT_BITS;
        }

        if carry > 0 {
            digits.push(carry as u32);
        }

        JSBI { sign: false, digits }.trim()
    }

    fn abs_sub(x: &JSBI, y: &JSBI) -> JSBI {
        // Assumes x >= y in absolute value
        let mut digits = Vec::with_capacity(x.digits.len());
        let mut borrow = 0i64;

        for i in 0..x.digits.len() {
            let dx = x.digits[i] as i64;
            let dy = *y.digits.get(i).unwrap_or(&0) as i64;
            let diff = dx - dy - borrow;
            if diff < 0 {
                digits.push((diff + (DIGIT_BASE as i64)) as u32);
                borrow = 1;
            } else {
                digits.push(diff as u32);
                borrow = 0;
            }
        }

        JSBI { sign: false, digits }.trim()
    }

    fn abs_mul(x: &JSBI, y: &JSBI) -> JSBI {
        if x.is_zero() || y.is_zero() {
            return JSBI::zero();
        }
        let x_len = x.digits.len();
        let y_len = y.digits.len();
        let mut digits = vec![0u32; x_len + y_len];

        for i in 0..x_len {
            let dx = x.digits[i] as u64;
            let mut carry = 0u64;
            for j in 0..y_len {
                let dy = y.digits[j] as u64;
                let current = digits[i + j] as u64 + dx * dy + carry;
                digits[i + j] = (current & (DIGIT_MASK as u64)) as u32;
                carry = current >> DIGIT_BITS;
            }
            let mut k = i + y_len;
            while carry > 0 {
                let sum = digits[k] as u64 + carry;
                digits[k] = (sum & (DIGIT_MASK as u64)) as u32;
                carry = sum >> DIGIT_BITS;
                k += 1;
            }
        }

        JSBI { sign: false, digits }.trim()
    }

    pub fn abs_div_rem(x: &JSBI, y: &JSBI) -> (JSBI, JSBI) {
        if y.is_zero() {
            panic!("Division by zero");
        }
        let mut abs_x = x.clone();
        abs_x.sign = false;
        let mut abs_y = y.clone();
        abs_y.sign = false;

        if JSBI::abs_cmp(&abs_x, &abs_y) == Ordering::Less {
            return (JSBI::zero(), abs_x);
        }
        if abs_y.digits.len() == 1 {
            let divisor = abs_y.digits[0] as u64;
            let mut q_digits = vec![0u32; abs_x.digits.len()];
            let mut rem = 0u64;
            for i in (0..abs_x.digits.len()).rev() {
                let cur = (rem << DIGIT_BITS) | (abs_x.digits[i] as u64);
                q_digits[i] = (cur / divisor) as u32;
                rem = cur % divisor;
            }
            let quotient = JSBI { sign: false, digits: q_digits }.trim();
            let remainder = if rem == 0 { JSBI::zero() } else { JSBI { sign: false, digits: vec![rem as u32] } };
            return (quotient, remainder);
        }

        // Knuth Algorithm D implementation for multi-digit division
        let n = abs_y.digits.len();
        let m = abs_x.digits.len();

        let v_top = abs_y.digits[n - 1] as u64;
        let d = (DIGIT_BASE / (v_top + 1)) as u32;

        let u_norm = JSBI::abs_mul(&abs_x, &JSBI::from_u64(d as u64));
        let v_norm = JSBI::abs_mul(&abs_y, &JSBI::from_u64(d as u64));

        let mut u = u_norm.digits;
        while u.len() <= m + 1 {
            u.push(0);
        }
        let v = v_norm.digits;
        let vn1 = v[n - 1] as u64;
        let vn2 = v[n - 2] as u64;

        let mut q = vec![0u32; m - n + 1];

        for j in (0..=(m - n)).rev() {
            let u_jn = u[j + n] as u64;
            let u_jn1 = u[j + n - 1] as u64;
            let u_jn2 = u[j + n - 2] as u64;

            let mut qhat = if u_jn == vn1 {
                DIGIT_MASK as u64
            } else {
                ((u_jn << DIGIT_BITS) | u_jn1) / vn1
            };

            let mut rhat = ((u_jn << DIGIT_BITS) | u_jn1) - qhat * vn1;

            while qhat >= DIGIT_BASE || (qhat * vn2 > ((rhat << DIGIT_BITS) | u_jn2)) {
                qhat -= 1;
                rhat += vn1;
                if rhat >= DIGIT_BASE {
                    break;
                }
            }

            let mut borrow = 0i64;
            for i in 0..n {
                let p = qhat * (v[i] as u64);
                let diff = (u[j + i] as i64) - (borrow + ((p & (DIGIT_MASK as u64)) as i64));
                u[j + i] = (diff.rem_euclid(DIGIT_BASE as i64)) as u32;
                borrow = (p >> DIGIT_BITS) as i64 - if diff < 0 { (diff + 1) / (DIGIT_BASE as i64) - 1 } else { diff / (DIGIT_BASE as i64) };
            }
            let diff = (u[j + n] as i64) - borrow;
            u[j + n] = (diff.rem_euclid(DIGIT_BASE as i64)) as u32;

            if diff < 0 {
                qhat -= 1;
                let mut carry = 0u64;
                for i in 0..n {
                    let sum = (u[j + i] as u64) + (v[i] as u64) + carry;
                    u[j + i] = (sum & (DIGIT_MASK as u64)) as u32;
                    carry = sum >> DIGIT_BITS;
                }
                u[j + n] = ((u[j + n] as u64) + carry) as u32;
            }

            q[j] = qhat as u32;
        }

        let quotient = JSBI { sign: false, digits: q }.trim();
        let u_rem = JSBI { sign: false, digits: u[0..n].to_vec() }.trim();
        let (remainder, _) = JSBI::abs_div_rem(&u_rem, &JSBI::from_u64(d as u64));

        (quotient, remainder)
    }

    // --- Core Public Arithmetic Methods ---

    pub fn add(x: &JSBI, y: &JSBI) -> JSBI {
        if x.is_zero() {
            return y.clone();
        }
        if y.is_zero() {
            return x.clone();
        }

        if x.sign == y.sign {
            let mut res = JSBI::abs_add(x, y);
            res.sign = x.sign;
            res
        } else {
            match JSBI::abs_cmp(x, y) {
                Ordering::Equal => JSBI::zero(),
                Ordering::Greater => {
                    let mut res = JSBI::abs_sub(x, y);
                    res.sign = x.sign;
                    res
                }
                Ordering::Less => {
                    let mut res = JSBI::abs_sub(y, x);
                    res.sign = y.sign;
                    res
                }
            }
        }
    }

    pub fn subtract(x: &JSBI, y: &JSBI) -> JSBI {
        let mut neg_y = y.clone();
        if !neg_y.is_zero() {
            neg_y.sign = !neg_y.sign;
        }
        JSBI::add(x, &neg_y)
    }

    pub fn multiply(x: &JSBI, y: &JSBI) -> JSBI {
        if x.is_zero() || y.is_zero() {
            return JSBI::zero();
        }
        let mut res = JSBI::abs_mul(x, y);
        res.sign = x.sign != y.sign;
        res
    }

    pub fn divide(x: &JSBI, y: &JSBI) -> Result<JSBI, JSBIError> {
        if y.is_zero() {
            return Err(JSBIError::RangeError("Division by zero".to_string()));
        }
        let (mut q, _) = JSBI::abs_div_rem(x, y);
        if !q.is_zero() {
            q.sign = x.sign != y.sign;
        }
        Ok(q)
    }

    pub fn remainder(x: &JSBI, y: &JSBI) -> Result<JSBI, JSBIError> {
        if y.is_zero() {
            return Err(JSBIError::RangeError("Division by zero".to_string()));
        }
        let (_, mut r) = JSBI::abs_div_rem(x, y);
        if !r.is_zero() {
            r.sign = x.sign;
        }
        Ok(r)
    }

    pub fn exponentiate(base: &JSBI, exp: &JSBI) -> Result<JSBI, JSBIError> {
        if exp.sign {
            return Err(JSBIError::RangeError("Exponent must be positive".to_string()));
        }
        if exp.is_zero() {
            return Ok(JSBI::from_u64(1));
        }
        if base.is_zero() {
            return Ok(JSBI::zero());
        }

        let mut result = JSBI::from_u64(1);
        let mut b = base.clone();
        let mut e = exp.clone();

        let two = JSBI::from_u64(2);

        while !e.is_zero() {
            let (q, r) = JSBI::abs_div_rem(&e, &two);
            if !r.is_zero() {
                result = JSBI::multiply(&result, &b);
            }
            b = JSBI::multiply(&b, &b);
            e = q;
        }

        Ok(result)
    }

    pub fn unary_minus(x: &JSBI) -> JSBI {
        if x.is_zero() {
            JSBI::zero()
        } else {
            let mut res = x.clone();
            res.sign = !res.sign;
            res
        }
    }

    // --- Comparison Methods ---

    pub fn compare(x: &JSBI, y: &JSBI) -> Ordering {
        if x.is_zero() && y.is_zero() {
            return Ordering::Equal;
        }
        if x.sign != y.sign {
            return if x.sign { Ordering::Less } else { Ordering::Greater };
        }
        let abs_cmp = JSBI::abs_cmp(x, y);
        if x.sign { abs_cmp.reverse() } else { abs_cmp }
    }

    pub fn equal(x: &JSBI, y: &JSBI) -> bool {
        JSBI::compare(x, y) == Ordering::Equal
    }

    pub fn less_than(x: &JSBI, y: &JSBI) -> bool {
        JSBI::compare(x, y) == Ordering::Less
    }

    pub fn less_than_or_equal(x: &JSBI, y: &JSBI) -> bool {
        JSBI::compare(x, y) != Ordering::Greater
    }

    pub fn greater_than(x: &JSBI, y: &JSBI) -> bool {
        JSBI::compare(x, y) == Ordering::Greater
    }

    pub fn greater_than_or_equal(x: &JSBI, y: &JSBI) -> bool {
        JSBI::compare(x, y) != Ordering::Less
    }

    // --- Bitwise Operations & Shifts ---

    pub fn bitwise_not(x: &JSBI) -> JSBI {
        // ~x = -(x + 1)
        let one = JSBI::from_u64(1);
        let added = JSBI::add(x, &one);
        JSBI::unary_minus(&added)
    }

    fn twos_complement_digit(x: &JSBI, i: usize, x_abs_sub1: &JSBI) -> u32 {
        if !x.sign {
            *x.digits.get(i).unwrap_or(&0)
        } else {
            let digit = *x_abs_sub1.digits.get(i).unwrap_or(&0);
            (!digit) & DIGIT_MASK
        }
    }

    pub fn bitwise_and(x: &JSBI, y: &JSBI) -> JSBI {
        let x_abs_sub1 = if x.sign { JSBI::abs_sub(x, &JSBI::from_u64(1)) } else { JSBI::zero() };
        let y_abs_sub1 = if y.sign { JSBI::abs_sub(y, &JSBI::from_u64(1)) } else { JSBI::zero() };

        let res_is_neg = x.sign && y.sign;
        let max_len = x.digits.len().max(y.digits.len()) + 1;
        let mut raw_digits = Vec::with_capacity(max_len);

        for i in 0..max_len {
            let dx = JSBI::twos_complement_digit(x, i, &x_abs_sub1);
            let dy = JSBI::twos_complement_digit(y, i, &y_abs_sub1);
            raw_digits.push(dx & dy);
        }

        if !res_is_neg {
            JSBI { sign: false, digits: raw_digits }.trim()
        } else {
            let mut inverted_digits = Vec::with_capacity(raw_digits.len());
            for &d in &raw_digits {
                inverted_digits.push((!d) & DIGIT_MASK);
            }
            let inverted = JSBI { sign: false, digits: inverted_digits }.trim();
            let added = JSBI::add(&inverted, &JSBI::from_u64(1));
            JSBI::unary_minus(&added)
        }
    }

    pub fn bitwise_or(x: &JSBI, y: &JSBI) -> JSBI {
        let x_abs_sub1 = if x.sign { JSBI::abs_sub(x, &JSBI::from_u64(1)) } else { JSBI::zero() };
        let y_abs_sub1 = if y.sign { JSBI::abs_sub(y, &JSBI::from_u64(1)) } else { JSBI::zero() };

        let res_is_neg = x.sign || y.sign;
        let max_len = x.digits.len().max(y.digits.len()) + 1;
        let mut raw_digits = Vec::with_capacity(max_len);

        for i in 0..max_len {
            let dx = JSBI::twos_complement_digit(x, i, &x_abs_sub1);
            let dy = JSBI::twos_complement_digit(y, i, &y_abs_sub1);
            raw_digits.push(dx | dy);
        }

        if !res_is_neg {
            JSBI { sign: false, digits: raw_digits }.trim()
        } else {
            let mut inverted_digits = Vec::with_capacity(raw_digits.len());
            for &d in &raw_digits {
                inverted_digits.push((!d) & DIGIT_MASK);
            }
            let inverted = JSBI { sign: false, digits: inverted_digits }.trim();
            let added = JSBI::add(&inverted, &JSBI::from_u64(1));
            JSBI::unary_minus(&added)
        }
    }

    pub fn bitwise_xor(x: &JSBI, y: &JSBI) -> JSBI {
        let x_abs_sub1 = if x.sign { JSBI::abs_sub(x, &JSBI::from_u64(1)) } else { JSBI::zero() };
        let y_abs_sub1 = if y.sign { JSBI::abs_sub(y, &JSBI::from_u64(1)) } else { JSBI::zero() };

        let res_is_neg = x.sign != y.sign;
        let max_len = x.digits.len().max(y.digits.len()) + 1;
        let mut raw_digits = Vec::with_capacity(max_len);

        for i in 0..max_len {
            let dx = JSBI::twos_complement_digit(x, i, &x_abs_sub1);
            let dy = JSBI::twos_complement_digit(y, i, &y_abs_sub1);
            raw_digits.push(dx ^ dy);
        }

        if !res_is_neg {
            JSBI { sign: false, digits: raw_digits }.trim()
        } else {
            let mut inverted_digits = Vec::with_capacity(raw_digits.len());
            for &d in &raw_digits {
                inverted_digits.push((!d) & DIGIT_MASK);
            }
            let inverted = JSBI { sign: false, digits: inverted_digits }.trim();
            let added = JSBI::add(&inverted, &JSBI::from_u64(1));
            JSBI::unary_minus(&added)
        }
    }

    pub fn left_shift(x: &JSBI, shift_amount: &JSBI) -> JSBI {
        if x.is_zero() {
            return JSBI::zero();
        }
        if shift_amount.sign {
            return JSBI::signed_right_shift(x, &JSBI::unary_minus(shift_amount));
        }

        let shift = shift_amount.to_number() as u64;
        let digit_shift = (shift / (DIGIT_BITS as u64)) as usize;
        let bit_shift = (shift % (DIGIT_BITS as u64)) as u32;

        let mut digits = vec![0u32; digit_shift];
        let mut carry = 0u64;

        for &d in &x.digits {
            let val = ((d as u64) << bit_shift) | carry;
            digits.push((val & (DIGIT_MASK as u64)) as u32);
            carry = val >> DIGIT_BITS;
        }

        if carry > 0 {
            digits.push(carry as u32);
        }

        JSBI { sign: x.sign, digits }.trim()
    }

    pub fn signed_right_shift(x: &JSBI, shift_amount: &JSBI) -> JSBI {
        if x.is_zero() {
            return JSBI::zero();
        }
        if shift_amount.sign {
            return JSBI::left_shift(x, &JSBI::unary_minus(shift_amount));
        }

        let shift = shift_amount.to_number() as u64;
        let digit_shift = (shift / (DIGIT_BITS as u64)) as usize;
        let bit_shift = (shift % (DIGIT_BITS as u64)) as u32;

        if digit_shift >= x.digits.len() {
            return if x.sign { JSBI::from_i64(-1) } else { JSBI::zero() };
        }

        if !x.sign {
            let mut digits = Vec::with_capacity(x.digits.len() - digit_shift);
            let mut carry = 0u32;

            for &d in x.digits[digit_shift..].iter().rev() {
                let val = (carry << (DIGIT_BITS - bit_shift)) | (d >> bit_shift);
                digits.push(val);
                carry = d & ((1 << bit_shift) - 1);
            }

            digits.reverse();
            JSBI { sign: false, digits }.trim()
        } else {
            // For negative numbers, right shift is floor(x / 2^shift)
            // x >> shift = ~((~x) >> shift)
            let not_x = JSBI::bitwise_not(x);
            let shifted_not_x = JSBI::signed_right_shift(&not_x, shift_amount);
            JSBI::bitwise_not(&shifted_not_x)
        }
    }

    // --- Bit Truncation Methods (asIntN & asUintN) ---

    pub fn as_uint_n(n: u32, x: &JSBI) -> JSBI {
        if n == 0 || x.is_zero() {
            return JSBI::zero();
        }
        if !x.sign {
            // Mask lower n bits
            let modulus = JSBI::left_shift(&JSBI::from_u64(1), &JSBI::from_u64(n as u64));
            let (_, rem) = JSBI::abs_div_rem(x, &modulus);
            rem
        } else {
            // Two's complement: (1 << n) + (x % (1 << n))
            let modulus = JSBI::left_shift(&JSBI::from_u64(1), &JSBI::from_u64(n as u64));
            let (_, rem) = JSBI::abs_div_rem(x, &modulus);
            if rem.is_zero() {
                JSBI::zero()
            } else {
                JSBI::subtract(&modulus, &rem)
            }
        }
    }

    pub fn as_int_n(n: u32, x: &JSBI) -> JSBI {
        if n == 0 {
            return JSBI::zero();
        }
        let uint_val = JSBI::as_uint_n(n, x);
        let bit_check = JSBI::left_shift(&JSBI::from_u64(1), &JSBI::from_u64((n - 1) as u64));
        if JSBI::greater_than_or_equal(&uint_val, &bit_check) {
            let modulus = JSBI::left_shift(&JSBI::from_u64(1), &JSBI::from_u64(n as u64));
            JSBI::subtract(&uint_val, &modulus)
        } else {
            uint_val
        }
    }

    // --- DataView Support ---

    pub fn data_view_get_big_int64(bytes: &[u8], little_endian: bool) -> Result<JSBI, JSBIError> {
        if bytes.len() < 8 {
            return Err(JSBIError::RangeError("DataView buffer too small".to_string()));
        }
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&bytes[..8]);
        let val = if little_endian {
            i64::from_le_bytes(arr)
        } else {
            i64::from_be_bytes(arr)
        };
        Ok(JSBI::from_i64(val))
    }

    pub fn to_u64(&self) -> u64 {
        let mut res = 0u64;
        if let Some(&d0) = self.digits.get(0) {
            res |= d0 as u64;
        }
        if let Some(&d1) = self.digits.get(1) {
            res |= (d1 as u64) << 30;
        }
        if let Some(&d2) = self.digits.get(2) {
            res |= ((d2 & 0xF) as u64) << 60;
        }
        res
    }

    pub fn to_i64(&self) -> i64 {
        let u = self.to_u64();
        if self.sign { -(u as i64) } else { u as i64 }
    }

    pub fn data_view_set_big_int64(bytes: &mut [u8], value: &JSBI, little_endian: bool) -> Result<(), JSBIError> {
        if bytes.len() < 8 {
            return Err(JSBIError::RangeError("DataView buffer too small".to_string()));
        }
        let truncated = JSBI::as_int_n(64, value);
        let num = truncated.to_i64();
        let encoded = if little_endian {
            num.to_le_bytes()
        } else {
            num.to_be_bytes()
        };
        bytes[..8].copy_from_slice(&encoded);
        Ok(())
    }

    pub fn data_view_get_big_uint64(bytes: &[u8], little_endian: bool) -> Result<JSBI, JSBIError> {
        if bytes.len() < 8 {
            return Err(JSBIError::RangeError("DataView buffer too small".to_string()));
        }
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&bytes[..8]);
        let val = if little_endian {
            u64::from_le_bytes(arr)
        } else {
            u64::from_be_bytes(arr)
        };
        Ok(JSBI::from_u64(val))
    }

    pub fn data_view_set_big_uint64(bytes: &mut [u8], value: &JSBI, little_endian: bool) -> Result<(), JSBIError> {
        if bytes.len() < 8 {
            return Err(JSBIError::RangeError("DataView buffer too small".to_string()));
        }
        let truncated = JSBI::as_uint_n(64, value);
        let num = truncated.to_u64();
        let encoded = if little_endian {
            num.to_le_bytes()
        } else {
            num.to_be_bytes()
        };
        bytes[..8].copy_from_slice(&encoded);
        Ok(())
    }
}

impl PartialEq for JSBI {
    fn eq(&self, other: &Self) -> bool {
        JSBI::equal(self, other)
    }
}

impl PartialOrd for JSBI {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(JSBI::compare(self, other))
    }
}

impl fmt::Display for JSBI {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string(10))
    }
}

impl fmt::Debug for JSBI {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BigInt[{:?}]", self.digits)
    }
}

// --- Trait Overloads for Idiomatic Rust ---

impl Add for &JSBI {
    type Output = JSBI;
    fn add(self, rhs: Self) -> JSBI {
        JSBI::add(self, rhs)
    }
}

impl Sub for &JSBI {
    type Output = JSBI;
    fn sub(self, rhs: Self) -> JSBI {
        JSBI::subtract(self, rhs)
    }
}

impl Mul for &JSBI {
    type Output = JSBI;
    fn mul(self, rhs: Self) -> JSBI {
        JSBI::multiply(self, rhs)
    }
}

impl Div for &JSBI {
    type Output = JSBI;
    fn div(self, rhs: Self) -> JSBI {
        JSBI::divide(self, rhs).unwrap()
    }
}

impl Rem for &JSBI {
    type Output = JSBI;
    fn rem(self, rhs: Self) -> JSBI {
        JSBI::remainder(self, rhs).unwrap()
    }
}

impl BitAnd for &JSBI {
    type Output = JSBI;
    fn bitand(self, rhs: Self) -> JSBI {
        JSBI::bitwise_and(self, rhs)
    }
}

impl BitOr for &JSBI {
    type Output = JSBI;
    fn bitor(self, rhs: Self) -> JSBI {
        JSBI::bitwise_or(self, rhs)
    }
}

impl BitXor for &JSBI {
    type Output = JSBI;
    fn bitxor(self, rhs: Self) -> JSBI {
        JSBI::bitwise_xor(self, rhs)
    }
}

impl Shl<&JSBI> for &JSBI {
    type Output = JSBI;
    fn shl(self, rhs: &JSBI) -> JSBI {
        JSBI::left_shift(self, rhs)
    }
}

impl Shr<&JSBI> for &JSBI {
    type Output = JSBI;
    fn shr(self, rhs: &JSBI) -> JSBI {
        JSBI::signed_right_shift(self, rhs)
    }
}

impl Neg for &JSBI {
    type Output = JSBI;
    fn neg(self) -> JSBI {
        JSBI::unary_minus(self)
    }
}

impl Not for &JSBI {
    type Output = JSBI;
    fn not(self) -> JSBI {
        JSBI::bitwise_not(self)
    }
}
