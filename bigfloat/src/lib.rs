// BigFloat arbitrary-precision floating-point library.
// Translated from C++ by bob-jenkins.

#[cfg(feature = "bigfloat-test")]
const C_DIGITS: usize = 4;
#[cfg(not(feature = "bigfloat-test"))]
const C_DIGITS: usize = 20;

#[cfg(feature = "bigfloat-test")]
const C_LOG: u32 = 2;
#[cfg(not(feature = "bigfloat-test"))]
const C_LOG: u32 = 32;

const C_RANGE: u64 = 1u64 << C_LOG;

#[cfg(feature = "bigfloat-test")]
const C_ZERO_EXPONENT: i64 = -(1i64 << 4);
#[cfg(not(feature = "bigfloat-test"))]
const C_ZERO_EXPONENT: i64 = -(1i64 << 62);

const C_MIN_EXPONENT: i64 = C_ZERO_EXPONENT + C_DIGITS as i64;
const C_MAX_EXPONENT: i64 = -C_ZERO_EXPONENT;

const E_POWER_NEG: i64 = -7; // = _ePowerNeg in C++

macro_rules! assert_bf {
    ($cond:expr) => { assert!($cond) };
    ($cond:expr, $fmt:literal $(, $arg:expr)*) => {
        if !$cond { panic!(concat!("assertion failed: ", $fmt) $(, $arg)*); }
    };
}

#[derive(Clone)]
pub struct BigFloat {
    exponent: i64,
    d: [u32; C_DIGITS],
    is_negative: bool,
    length: u16,
}

impl Default for BigFloat {
    fn default() -> Self {
        let mut bf = BigFloat {
            exponent: 0,
            d: [0u32; C_DIGITS],
            is_negative: false,
            length: 0,
        };
        bf.p_zero();
        bf
    }
}

impl BigFloat {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_int(n: i64) -> Self {
        let mut bf = BigFloat::new();
        bf.from_integer(n, 0);
        bf
    }

    pub fn from_int_exp(n: i64, exponent: i64) -> Self {
        let mut bf = BigFloat::new();
        bf.from_integer(n, exponent);
        bf
    }

    // -----------------------------------------------------------------------
    // Special values
    // -----------------------------------------------------------------------

    pub fn p_zero(&mut self) -> &mut Self {
        self.exponent = C_ZERO_EXPONENT;
        self.length = 0;
        self.is_negative = false;
        self
    }

    pub fn n_zero(&mut self) -> &mut Self {
        self.exponent = C_ZERO_EXPONENT;
        self.length = 0;
        self.is_negative = true;
        self
    }

    pub fn zero(&mut self, neg: bool) -> &mut Self {
        if neg { self.n_zero() } else { self.p_zero() }
    }

    pub fn p_inf(&mut self) -> &mut Self {
        self.exponent = C_ZERO_EXPONENT;
        self.length = 1;
        self.is_negative = false;
        self
    }

    pub fn n_inf(&mut self) -> &mut Self {
        self.exponent = C_ZERO_EXPONENT;
        self.length = 1;
        self.is_negative = true;
        self
    }

    pub fn inf(&mut self, negative: bool) -> &mut Self {
        if negative { self.n_inf() } else { self.p_inf() }
    }

    pub fn nan(&mut self) -> &mut Self {
        self.exponent = C_ZERO_EXPONENT;
        self.length = 2;
        self.is_negative = false;
        self
    }

    // -----------------------------------------------------------------------
    // State queries
    // -----------------------------------------------------------------------

    pub fn is_zero(&self) -> bool {
        self.exponent == C_ZERO_EXPONENT && self.length == 0
    }

    pub fn is_p_zero(&self) -> bool {
        self.exponent == C_ZERO_EXPONENT && self.length == 0 && !self.is_negative
    }

    pub fn is_n_zero(&self) -> bool {
        self.exponent == C_ZERO_EXPONENT && self.length == 0 && self.is_negative
    }

    pub fn is_inf(&self) -> bool {
        self.exponent == C_ZERO_EXPONENT && self.length == 1
    }

    pub fn is_p_inf(&self) -> bool {
        self.exponent == C_ZERO_EXPONENT && self.length == 1 && !self.is_negative
    }

    pub fn is_n_inf(&self) -> bool {
        self.exponent == C_ZERO_EXPONENT && self.length == 1 && self.is_negative
    }

    pub fn is_nan(&self) -> bool {
        self.exponent == C_ZERO_EXPONENT && self.length == 2
    }

    pub fn is_special(&self) -> bool {
        self.exponent == C_ZERO_EXPONENT
    }

    pub fn is_negative(&self) -> bool {
        self.is_negative
    }

    pub fn to_exponent(&self) -> i64 {
        self.exponent
    }

    // -----------------------------------------------------------------------
    // Copy / negate
    // -----------------------------------------------------------------------

    pub fn copy_from(&mut self, n: &BigFloat) -> &mut Self {
        self.exponent = n.exponent;
        self.is_negative = n.is_negative;
        self.length = n.length;
        let len = self.length as usize;
        for i in 0..len {
            self.d[i] = n.d[i];
        }
        self
    }

    pub fn negate(&mut self) -> &mut Self {
        self.is_negative = !self.is_negative;
        self
    }

    // -----------------------------------------------------------------------
    // Conversion from/to integer
    // -----------------------------------------------------------------------

    pub fn from_integer(&mut self, orig_num: i64, exponent: i64) -> &mut Self {
        if orig_num == 0 {
            return self.zero(false);
        }

        self.is_negative = orig_num < 0;
        let mut num: u64 = if self.is_negative {
            (-(orig_num as i128)) as u64
        } else {
            orig_num as u64
        };

        let shift = 64 - C_LOG;
        self.exponent = (shift / C_LOG) as i64;
        let mask: u64 = (C_RANGE - 1) << shift;
        while (num & mask) == 0 {
            self.exponent -= 1;
            num = num.wrapping_shl(C_LOG);
        }

        self.length = 0;
        self.exponent += exponent;
        while num != 0 && (self.length as usize) < C_DIGITS {
            self.d[self.length as usize] = ((num & mask) >> shift) as u32;
            self.length += 1;
            num = num.wrapping_shl(C_LOG);
        }

        self
    }

    pub fn to_integer(&self) -> i64 {
        if self.is_zero() {
            return 0;
        }

        assert_bf!(
            self.exponent < ((64 - C_LOG as i64) / C_LOG as i64)
                || (self.exponent == (64 - C_LOG as i64) / C_LOG as i64
                    && self.d[0] < (C_RANGE / 2) as u32),
            "exponent unsuitable for integer conversion"
        );

        let mut result: i64 = 0;
        let mut pos: usize = 0;
        while pos < self.length as usize {
            result <<= C_LOG;
            result += self.d[pos] as i64;
            pos += 1;
        }
        let shift = C_LOG as i64 * (self.exponent + 1 - self.length as i64);
        if shift > 0 {
            result <<= shift;
        } else if shift < 0 {
            result >>= -shift;
        }
        if self.is_negative {
            result = -result;
        }
        result
    }

    pub fn to_digits(&self) -> u64 {
        let mut result: u64 = 0;
        let mut pos = self.length as usize;
        while pos > 0 {
            pos -= 1;
            result >>= C_LOG;
            result = result.wrapping_add((self.d[pos] as u64).wrapping_shl(64 - C_LOG));
        }
        result
    }

    pub fn to_double(&self) -> f64 {
        if self.is_special() {
            if self.is_zero() {
                return if self.is_negative { -0.0f64 } else { 0.0f64 };
            } else if self.is_inf() {
                return if self.is_negative {
                    f64::NEG_INFINITY
                } else {
                    f64::INFINITY
                };
            } else {
                return f64::NAN;
            }
        }

        let mut x = 0.0f64;
        let range_f = C_RANGE as f64;

        let len = self.length as usize;
        let mut i = len;
        while i > 0 {
            i -= 1;
            x /= range_f;
            x += self.d[i] as f64;
        }

        let mut i = 0i64;
        while i < self.exponent {
            x *= range_f;
            i += 1;
        }
        while i > self.exponent {
            x /= range_f;
            i -= 1;
        }

        if self.is_negative {
            x = -x;
        }
        x
    }

    // -----------------------------------------------------------------------
    // Rounding
    // -----------------------------------------------------------------------

    // Round after subtraction (previousDigit may be negative)
    pub fn round_signed(&mut self, carry: bool, previous_digit: i64) -> &mut Self {
        let mut ipos: i64;
        let mut round: bool;
        let mut carry = carry;
        let mut previous_digit = previous_digit;

        assert_bf!(previous_digit == 0 || self.length as usize == C_DIGITS);

        if -previous_digit > (C_RANGE as i64) / 2 {
            ipos = C_DIGITS as i64 - 1;
            round = true;
            previous_digit += C_RANGE as i64;
            while round && ipos >= 0 {
                let sum = self.d[ipos as usize] as i64 - round as i64;
                round = sum < 0;
                let sum = if round { sum + C_RANGE as i64 } else { sum };
                self.d[ipos as usize] = sum as u32;
                ipos -= 1;
            }
            if round {
                assert_bf!(carry);
                carry = false;
            }
        }

        if carry {
            let mut round2 = false;
            if self.length as usize == C_DIGITS {
                ipos = C_DIGITS as i64 - 1;
                round2 = self.d[ipos as usize] > (C_RANGE / 2) as u32
                    || (self.d[ipos as usize] == (C_RANGE / 2) as u32 && previous_digit >= 0);
                ipos -= 1;
                while round2 && ipos >= 0 {
                    let sum = self.d[ipos as usize] as u64 + round2 as u64;
                    round2 = sum >= C_RANGE;
                    let sum = if round2 { sum - C_RANGE } else { sum };
                    self.d[ipos as usize] = sum as u32;
                    ipos -= 1;
                }
            } else {
                self.length += 1;
            }

            // shift everyone over by one
            let len = self.length as usize;
            let mut ip = len;
            while ip > 1 {
                self.d[ip - 1] = self.d[ip - 2];
                ip -= 1;
            }

            // add carry as top new digit
            self.d[0] = (carry as u64 + round2 as u64) as u32;

            // increase the exponent by one
            if self.exponent == C_MAX_EXPONENT {
                let neg = self.is_negative;
                return self.inf(neg);
            }
            self.exponent += 1;
        } else if previous_digit >= (C_RANGE as i64) / 2 {
            assert_bf!(self.length as usize == C_DIGITS);
            round = true;
            ipos = C_DIGITS as i64 - 1;
            while round && ipos >= 0 {
                let sum = self.d[ipos as usize] as u64 + round as u64;
                round = sum >= C_RANGE;
                let sum = if round { sum - C_RANGE } else { sum };
                self.d[ipos as usize] = sum as u32;
                ipos -= 1;
            }
            if round {
                self.length = 1;
                self.d[0] = 1;
                if self.exponent == C_MAX_EXPONENT {
                    let neg = self.is_negative;
                    return self.inf(neg);
                }
                self.exponent += 1;
            }
        }

        // remove trailing zeros
        if self.length > 0 {
            while self.d[self.length as usize - 1] == 0 {
                self.length -= 1;
                if self.length == 0 {
                    let neg = self.is_negative;
                    self.zero(neg);
                    break;
                }
            }
        }

        // final postchecks
        if self.exponent < C_MIN_EXPONENT {
            let neg = self.is_negative;
            return self.zero(neg);
        } else if self.exponent > C_MAX_EXPONENT {
            let neg = self.is_negative;
            return self.inf(neg);
        }

        self
    }

    // Round after mult/div (previousDigit always positive)
    pub fn round_unsigned(&mut self, previous_digit: u64) -> &mut Self {
        let mut ipos: i64;
        let mut round: bool;

        assert_bf!(previous_digit == 0 || self.length as usize == C_DIGITS);

        if previous_digit >= C_RANGE / 2 {
            assert_bf!(self.length as usize == C_DIGITS);
            round = true;
            ipos = C_DIGITS as i64 - 1;
            while round && ipos >= 0 {
                let sum = self.d[ipos as usize] as u64 + round as u64;
                round = sum >= C_RANGE;
                let sum = if round { sum - C_RANGE } else { sum };
                self.d[ipos as usize] = sum as u32;
                ipos -= 1;
            }
            if round {
                self.length = 1;
                self.d[0] = 1;
                if self.exponent == C_MAX_EXPONENT {
                    let neg = self.is_negative;
                    return self.inf(neg);
                }
                self.exponent += 1;
            }
        }

        // remove trailing zeros
        while self.length > 0 && self.d[self.length as usize - 1] == 0 {
            self.length -= 1;
            if self.length == 0 {
                let neg = self.is_negative;
                self.zero(neg);
                break;
            }
        }

        // final postchecks
        if self.exponent < C_MIN_EXPONENT {
            let neg = self.is_negative;
            return self.zero(neg);
        } else if self.exponent > C_MAX_EXPONENT {
            let neg = self.is_negative;
            return self.inf(neg);
        }

        self
    }

    // -----------------------------------------------------------------------
    // Truncate
    // -----------------------------------------------------------------------

    pub fn trunc(&mut self) -> &mut Self {
        let diff = self.exponent + 1 - self.length as i64;
        if diff >= 0 {
            // already an integer (or zero fractional part)
        } else if self.exponent < 0 {
            self.zero(self.is_negative);
        } else {
            self.length = (self.exponent + 1) as u16;
        }
        self
    }

    // -----------------------------------------------------------------------
    // Comparison
    // -----------------------------------------------------------------------

    pub fn compare_absolute(&self, n: &BigFloat) -> i32 {
        if self.is_special() || n.is_special() {
            if self.is_inf() {
                return if n.is_inf() { 0 } else { 1 };
            } else if n.is_inf() {
                return -1;
            } else if self.is_zero() {
                return if n.is_zero() { 0 } else { -1 };
            } else if n.is_zero() {
                return 1;
            } else {
                return -1;
            }
        } else if self.exponent > n.exponent {
            return 1;
        } else if self.exponent < n.exponent {
            return -1;
        } else {
            // same exponent
            let length = self.length.min(n.length) as usize;
            for i in 0..length {
                let ad = self.d[i];
                let bd = n.d[i];
                if ad > bd {
                    return 1;
                } else if ad < bd {
                    return -1;
                }
            }
            if self.length > n.length {
                return 1;
            } else if self.length < n.length {
                return -1;
            }
        }
        0
    }

    pub fn compare(&self, n: &BigFloat) -> i32 {
        if self.is_negative != n.is_negative {
            return if self.is_negative { -1 } else { 1 };
        } else if self.is_negative {
            return -self.compare_absolute(n);
        } else {
            return self.compare_absolute(n);
        }
    }

    pub fn compare_int(&self, n: i64, exponent: i64) -> i32 {
        let b = BigFloat::from_int_exp(n, exponent);
        self.compare(&b)
    }

    // -----------------------------------------------------------------------
    // Add / subtract
    // -----------------------------------------------------------------------

    pub fn add(&mut self, n: &BigFloat) -> &mut Self {
        self.add_or_subtract(n, false)
    }

    pub fn sub(&mut self, n: &BigFloat) -> &mut Self {
        self.add_or_subtract(n, true)
    }

    pub fn add_int(&mut self, n: i64, exponent: i64) -> &mut Self {
        let b = BigFloat::from_int_exp(n, exponent);
        self.add(&b)
    }

    pub fn sub_int(&mut self, n: i64, exponent: i64) -> &mut Self {
        let b = BigFloat::from_int_exp(n, exponent);
        self.sub(&b)
    }

    fn add_or_subtract(&mut self, n: &BigFloat, minus: bool) -> &mut Self {
        // Clone self so we can use it as 'a' or 'b' safely
        let m = self.clone();

        if self.is_special() || n.is_special() {
            let mut n2 = n.clone();
            if minus {
                n2.negate();
            }
            if n2.is_zero() {
                return self;
            } else if self.is_zero() {
                return self.copy_from(&n2);
            } else if self.is_nan() || n.is_nan() {
                return self.nan();
            } else if self.is_p_inf() {
                return if n2.is_n_inf() { self.nan() } else { self };
            } else if self.is_n_inf() {
                return if n2.is_p_inf() { self.nan() } else { self };
            } else if n2.is_p_inf() {
                return self.p_inf();
            } else if n2.is_n_inf() {
                return self.n_inf();
            } else {
                panic!("add_or_subtract: unexpected special case");
            }
        }

        // Determine which of (m, n) has larger absolute value; that one is 'a'
        let use_n_as_a = self.compare_absolute(n) < 0;
        let sign_a: bool;
        let sign_b: bool;
        if use_n_as_a {
            // After swap: new signA = old signB = n.is_negative ^ minus
            //             new signB = old signA = m.is_negative
            sign_a = n.is_negative ^ minus;
            sign_b = m.is_negative;
        } else {
            sign_a = m.is_negative;
            sign_b = n.is_negative ^ minus;
        }

        let add = sign_a == sign_b;
        self.is_negative = sign_a;

        let (a, b): (&BigFloat, &BigFloat) = if use_n_as_a { (n, &m) } else { (&m, n) };

        let delta: i64 = a.exponent - b.exponent;
        if delta > C_DIGITS as i64 + 1 {
            // b is too small to matter; just copy a
            self.copy_from(a);
            self.is_negative = sign_a;
            return self;
        }

        let mut previous_digit: i64 = 0;
        self.exponent = a.exponent;
        let mut carry = false;
        let mut shift = false;

        // "least" positions mean (length), i.e., one past the last digit index
        let least_a = a.length as i64;
        let least_b = b.length as i64 + delta;

        // iPos tracks which digit position we're working on
        let mut ipos: i64;

        if least_a > least_b {
            // a has more digits; start by copying the low-order digits of a
            ipos = least_a - 1;
            self.length = least_a as u16;
            while ipos >= least_b {
                self.d[ipos as usize] = a.d[ipos as usize];
                ipos -= 1;
            }
            // Now ipos = least_b - 1; fall through to the main loop below
        } else {
            // b has >= as many significant positions; decide whether to shift
            if !add && least_b > C_DIGITS as i64 && a.d[0] == 1 {
                shift = true;
                self.exponent -= 1;
            }

            // Adjust for shift
            let least_b_work = if shift { least_b - 1 } else { least_b };
            let least_a_work = if shift { least_a - 1 } else { least_a };
            let delta_work   = if shift { delta   - 1 } else { delta };

            let least_b_used: i64 = if least_b_work > C_DIGITS as i64 {
                C_DIGITS as i64
            } else {
                least_b_work
            };

            ipos = least_b_used - 1;
            self.length = least_b_used as u16;

            // Handle the portion where only b has digits (ipos >= least_a_work)
            if ipos >= least_a_work {
                let stop_b = least_a_work.max(delta_work);
                if add {
                    while ipos >= stop_b {
                        let bd_idx = (ipos - delta_work) as usize;
                        let sum = b.d[bd_idx] as u64 + carry as u64;
                        carry = carry && (b.d[bd_idx] == 0); // carry propagates only through 0s when adding
                        self.d[ipos as usize] = sum as u32;
                        ipos -= 1;
                    }
                } else {
                    while ipos >= stop_b {
                        let bd_idx = (ipos - delta_work) as usize;
                        // the low-order digit is nonzero and carry==false initially,
                        // so it will be in range; c_range - x - carry
                        self.d[ipos as usize] = (C_RANGE - b.d[bd_idx] as u64 - carry as u64) as u32;
                        carry = true;
                        ipos -= 1;
                    }
                }

                // Handle the gap between where b ends and where a starts (if any)
                if stop_b > least_a_work {
                    if carry {
                        if add {
                            while ipos >= least_a_work {
                                self.d[ipos as usize] = 1;
                                carry = false;
                                ipos -= 1;
                            }
                        } else {
                            // subtraction borrow fills gap with 0xff..ff
                            while ipos >= least_a_work {
                                self.d[ipos as usize] = (C_RANGE - 1) as u32;
                                ipos -= 1;
                            }
                        }
                    } else {
                        while ipos >= least_a_work {
                            self.d[ipos as usize] = 0;
                            ipos -= 1;
                        }
                    }
                }
            }
            // At this point ipos == least_a_work - 1

            // Re-adjust ipos and do the main a+b overlap region
            // The C++ code asserts iPos == leastA-1 before the main loops.
            // With shift adjustment, leastA was already adjusted above.
            // However the main loops below use the *original* least_a / least_b / delta,
            // so we need to restore ipos to the unadjusted coordinate.
            // Actually the C++ code re-uses the same ipos variable, which at this point
            // equals leastA - 1 (after all the adjustments).  The main loops then use
            // the *original* delta (not delta_work) because `shift` has already been
            // applied to the exponent and the b-only region.  Let me look at the C++ again:
            //
            //   ASSERT(iPos == leastA-1);
            //   if (add) { while (iPos-delta >= 0) { ... a._d[iPos] + b._d[iPos-delta] ... } }
            //   else     { while (iPos-delta >= 0) { ... a._d[iPos+shift] - b._d[iPos-delta] ... } }
            //
            // So the main loops use `delta` (original, not delta_work) and `shift` as an
            // index offset into a.  `ipos` here == least_a_work - 1, not least_a - 1.
            // They are the same when !shift; when shift they differ by 1.
            // But shift is only set in the `else` branch (leastA <= leastB), not the `if` branch,
            // so within this else block ipos == least_a_work - 1.
            // The assert in C++ checks leastA-1 (original), but with shift, leastA was decremented
            // too... Actually in C++ the code modifies leastA, leastB, delta directly when shift=true.
            // I used separate _work variables. So ipos here is already at least_a_work - 1
            // which corresponds to the C++ leastA-1 after its in-place modification.
            // The C++ main loops then use `delta` (which it also modified in-place).
            // So I should use delta_work and least_a_work in the assertion and main loops.
            assert_bf!(ipos == least_a_work - 1,
                "ipos={} least_a_work-1={}", ipos, least_a_work - 1);

            if add {
                while ipos - delta_work >= 0 {
                    let sum = a.d[ipos as usize] as u64
                        + b.d[(ipos - delta_work) as usize] as u64
                        + carry as u64;
                    carry = sum >= C_RANGE;
                    self.d[ipos as usize] = (if carry { sum - C_RANGE } else { sum }) as u32;
                    ipos -= 1;
                }
                while ipos >= 0 {
                    let sum = a.d[ipos as usize] as u64 + carry as u64;
                    carry = sum >= C_RANGE;
                    self.d[ipos as usize] = (if carry { sum - C_RANGE } else { sum }) as u32;
                    ipos -= 1;
                }
                if least_b_work >= C_DIGITS as i64 + 1 {
                    let b_idx = C_DIGITS as i64 - delta_work;
                    previous_digit = if b_idx >= 0 { b.d[b_idx as usize] as i64 } else { 0 };
                }
            } else {
                while ipos - delta_work >= 0 {
                    let a_idx = (ipos + shift as i64) as usize;
                    let sum = a.d[a_idx] as i64
                        - b.d[(ipos - delta_work) as usize] as i64
                        - carry as i64;
                    carry = sum < 0;
                    self.d[ipos as usize] = (if carry { sum + C_RANGE as i64 } else { sum }) as u32;
                    ipos -= 1;
                }
                while ipos >= 0 {
                    let a_idx = (ipos + shift as i64) as usize;
                    let sum = a.d[a_idx] as i64 - carry as i64;
                    carry = sum < 0;
                    self.d[ipos as usize] = (if carry { sum + C_RANGE as i64 } else { sum }) as u32;
                    ipos -= 1;
                }
                if least_b_work >= C_DIGITS as i64 + 1 {
                    let b_idx = C_DIGITS as i64 - delta_work;
                    let b_digit = if b_idx >= 0 { b.d[b_idx as usize] as i64 } else { 0 };
                    previous_digit = -b_digit;
                    if least_b_work > C_DIGITS as i64 + 1 {
                        previous_digit -= 1;
                    }
                }
                if shift {
                    carry = !carry;
                } else {
                    assert_bf!(!carry);
                }
                if !carry {
                    let mut pos: usize = 0;
                    while pos < self.length as usize {
                        if self.d[pos] > 0 { break; }
                        pos += 1;
                    }
                    let adjust = pos as u16;
                    if adjust == self.length || self.exponent - (adjust as i64) < -C_MAX_EXPONENT {
                        let neg = self.is_negative;
                        return self.zero(neg);
                    } else if adjust > 0 {
                        self.exponent -= adjust as i64;
                        self.length -= adjust;
                        let new_len = self.length as usize;
                        for p in 0..new_len {
                            self.d[p] = self.d[p + adjust as usize];
                        }
                    }
                }
            }
            return self.round_signed(carry, previous_digit);
        }

        // ---- leastA > leastB branch: main loops (same as above else, but without shift) ----
        // Here shift is always false. ipos is currently least_b - 1.
        // The C++ code falls through from the initial copy into a single set of loops.
        if add {
            while ipos - delta >= 0 {
                let sum = a.d[ipos as usize] as u64
                    + b.d[(ipos - delta) as usize] as u64
                    + carry as u64;
                carry = sum >= C_RANGE;
                self.d[ipos as usize] = (if carry { sum - C_RANGE } else { sum }) as u32;
                ipos -= 1;
            }
            while ipos >= 0 {
                let sum = a.d[ipos as usize] as u64 + carry as u64;
                carry = sum >= C_RANGE;
                self.d[ipos as usize] = (if carry { sum - C_RANGE } else { sum }) as u32;
                ipos -= 1;
            }
            if least_b >= C_DIGITS as i64 + 1 {
                previous_digit = b.d[(C_DIGITS as i64 - delta) as usize] as i64;
            }
        } else {
            while ipos - delta >= 0 {
                let sum = a.d[ipos as usize] as i64
                    - b.d[(ipos - delta) as usize] as i64
                    - carry as i64;
                carry = sum < 0;
                self.d[ipos as usize] = (if carry { sum + C_RANGE as i64 } else { sum }) as u32;
                ipos -= 1;
            }
            while ipos >= 0 {
                let sum = a.d[ipos as usize] as i64 - carry as i64;
                carry = sum < 0;
                self.d[ipos as usize] = (if carry { sum + C_RANGE as i64 } else { sum }) as u32;
                ipos -= 1;
            }
            if least_b >= C_DIGITS as i64 + 1 {
                previous_digit = -(b.d[(C_DIGITS as i64 - delta) as usize] as i64);
                if least_b > C_DIGITS as i64 + 1 {
                    previous_digit -= 1;
                }
            }
            // shift is false here
            assert_bf!(!carry);
            if !carry {
                let mut pos: usize = 0;
                while pos < self.length as usize {
                    if self.d[pos] > 0 { break; }
                    pos += 1;
                }
                let adjust = pos as u16;
                if adjust == self.length || self.exponent - (adjust as i64) < -C_MAX_EXPONENT {
                    let neg = self.is_negative;
                    return self.zero(neg);
                } else if adjust > 0 {
                    self.exponent -= adjust as i64;
                    self.length -= adjust;
                    let new_len = self.length as usize;
                    for p in 0..new_len {
                        self.d[p] = self.d[p + adjust as usize];
                    }
                }
            }
        }

        self.round_signed(carry, previous_digit)
    }

    // -----------------------------------------------------------------------
    // Multiply
    // -----------------------------------------------------------------------

    pub fn mult(&mut self, n: &BigFloat) -> &mut Self {
        let exponent = self.exponent + n.exponent + 1;
        self.is_negative ^= n.is_negative;

        if self.is_special() || n.is_special() {
            if self.is_nan() || n.is_nan() {
                return self.nan();
            } else if self.is_zero() || n.is_zero() {
                let neg = self.is_negative;
                return self.zero(neg);
            } else {
                let neg = self.is_negative;
                return self.inf(neg);
            }
        } else if exponent < C_MIN_EXPONENT {
            let neg = self.is_negative;
            return self.zero(neg);
        } else if exponent > C_MAX_EXPONENT + 1 {
            let neg = self.is_negative;
            return self.inf(neg);
        }

        let mut length = self.length as u64 + n.length as u64;
        if length > C_DIGITS as u64 {
            length = C_DIGITS as u64 + 3;
        }

        let mut temp = [0u64; 2 * C_DIGITS + 3]; // enough for length <= C_DIGITS+3
        let len = length as usize;
        for i in 0..len {
            temp[i] = 0;
        }

        for i in (0..self.length as usize).rev() {
            let start = if n.length as u64 > length - i as u64 {
                (length - i as u64) as usize
            } else {
                n.length as usize
            };
            for j in (0..start).rev() {
                let result = self.d[i] as u64 * n.d[j] as u64;
                temp[i + j + 1] = temp[i + j + 1].wrapping_add(result & (C_RANGE - 1));
                temp[i + j] = temp[i + j].wrapping_add(result >> C_LOG);
            }
        }

        // carries
        let mut i = len;
        while i > 1 {
            i -= 1;
            temp[i - 1] = temp[i - 1].wrapping_add(temp[i] >> C_LOG);
            temp[i] &= C_RANGE - 1;
        }
        assert_bf!(len > 0 && temp[0] < C_RANGE);

        let mut result_offset: usize = 0;
        let mut exponent = exponent;
        if temp[0] == 0 {
            result_offset = 1;
            exponent -= 1;
        }
        let mut length = len as u64;
        if result_offset == 1 {
            length -= 1;
        }
        let mut previous_digit: u64 = 0;
        if length > C_DIGITS as u64 {
            length = C_DIGITS as u64;
            previous_digit = temp[result_offset + length as usize];
        }

        let len2 = length as usize;
        for i in 0..len2 {
            self.d[i] = temp[result_offset + i] as u32;
        }
        self.length = len2 as u16;
        self.exponent = exponent;

        self.round_unsigned(previous_digit)
    }

    pub fn mult_int(&mut self, n: i64, exponent: i64) -> &mut Self {
        let b = BigFloat::from_int_exp(n, exponent);
        self.mult(&b)
    }

    // -----------------------------------------------------------------------
    // Divide
    // -----------------------------------------------------------------------

    pub fn div(&mut self, n: &BigFloat) -> &mut Self {
        let exponent = self.exponent - n.exponent;
        self.is_negative = self.is_negative ^ n.is_negative;

        if self.is_special() || n.is_special() {
            if self.is_nan() || n.is_nan() || n.is_zero() {
                return self.nan();
            } else if self.is_zero() || n.is_inf() {
                let neg = self.is_negative;
                return self.zero(neg);
            } else {
                let neg = self.is_negative;
                return self.inf(neg);
            }
        } else if exponent + 1 < C_MIN_EXPONENT {
            let neg = self.is_negative;
            return self.zero(neg);
        } else if exponent > C_MAX_EXPONENT + 1 {
            let neg = self.is_negative;
            return self.inf(neg);
        }

        assert_bf!((C_LOG / 2) * 2 == C_LOG);
        assert_bf!(self.length > 0);

        let limit: usize = 2 * (C_DIGITS + 2);
        let numer_limit: usize = 2 * limit + 2;

        let half_log = C_LOG / 2;
        let half_mask: i64 = (1i64 << half_log) - 1;
        let half_range = 1i64 << half_log;

        // numerator in half-digits
        let mut t = vec![0i64; numer_limit];
        for i in 0..self.length as usize {
            t[2 * i] = (self.d[i] >> half_log) as i64;
            t[2 * i + 1] = (self.d[i] as i64) & half_mask;
        }

        // denominator in half-digits
        let mut d_arr = vec![0i64; limit];
        let mut ad = n.d[0] as i64;
        let mut length = 2 * n.length as i64;
        let mut shift = if ad < half_range { 1i64 } else { 0i64 };
        if shift != 0 {
            length -= 1;
            d_arr[0] = ad;
            ad <<= half_log;
            let mut ii = 2usize;
            while ii < 2 * n.length as usize {
                d_arr[ii - 1] = (n.d[ii / 2] >> half_log) as i64;
                d_arr[ii] = (n.d[ii / 2] as i64) & half_mask;
                ii += 2;
            }
            if n.length > 1 {
                ad += d_arr[1];
            }
        } else {
            let mut ii = 0usize;
            while ii < 2 * n.length as usize {
                d_arr[ii] = (n.d[ii / 2] >> half_log) as i64;
                d_arr[ii + 1] = (n.d[ii / 2] as i64) & half_mask;
                ii += 2;
            }
        }

        // divide
        let mut r = vec![0i64; limit];
        let mut an = t[0];
        for i in 0..limit {
            an = (an << half_log) + t[i + 1];
            let q = an / ad;
            if q != 0 {
                if 2 < length {
                    t[i + 2] -= q * d_arr[2];
                    let mut j = length as usize;
                    while j > 3 {
                        j -= 1;
                        let p = t[i + j] - q * d_arr[j];
                        assert_bf!(
                            p < (1i64 << (3 * half_log)) && p > -(1i64 << (3 * half_log))
                        );
                        t[i + j - 1] += p >> half_log;
                        t[i + j] = p & half_mask;
                    }
                }
            }
            assert_bf!(ad * q + (an % ad) == an);
            an %= ad;
            r[i] = q;
        }

        // carry
        let mut i = limit;
        while i > 1 {
            i -= 1;
            let carry = r[i] >> half_log;
            if carry != 0 {
                r[i] &= half_mask;
                r[i - 1] += carry;
            }
        }
        assert_bf!((r[0] >> half_log) == 0);

        let mut pr_offset: usize = 0;
        while r[pr_offset] == 0 {
            pr_offset += 1;
            shift -= 1;
        }

        // combine half-digits and round
        let previous_digit: i64;
        if (shift & 1) == 0 {
            self.d[0] = r[pr_offset] as u32;
            for i in 1..C_DIGITS {
                self.d[i] = ((r[pr_offset + 2 * i - 1] << half_log)
                    + r[pr_offset + 2 * i]) as u32;
            }
            previous_digit = (r[pr_offset + 2 * C_DIGITS - 1] << half_log)
                + r[pr_offset + 2 * C_DIGITS];
        } else {
            for i in 0..C_DIGITS {
                self.d[i] =
                    ((r[pr_offset + 2 * i] << half_log) + r[pr_offset + 2 * i + 1]) as u32;
            }
            previous_digit = (r[pr_offset + 2 * C_DIGITS] << half_log)
                + r[pr_offset + 2 * C_DIGITS + 1];
        }
        self.length = C_DIGITS as u16;
        self.exponent = exponent + (shift >> 1);
        self.round_unsigned(previous_digit as u64)
    }

    pub fn div_int(&mut self, n: i64, exponent: i64) -> &mut Self {
        let b = BigFloat::from_int_exp(n, exponent);
        self.div(&b)
    }

    // -----------------------------------------------------------------------
    // Invert
    // -----------------------------------------------------------------------

    pub fn invert(&mut self) -> &mut Self {
        let mut i = BigFloat::from_int(1);
        i.div(self);
        self.copy_from(&i)
    }

    // -----------------------------------------------------------------------
    // Sqrt
    // -----------------------------------------------------------------------

    pub fn sqrt(&mut self) -> &mut Self {
        if self.is_negative {
            return self.nan();
        } else if self.is_special() {
            return self;
        }

        const C_INTERMEDIATE: usize = 2 * C_DIGITS + 2;
        const C_REMAINDER: usize = 2 * C_INTERMEDIATE;

        let mut r = [0i64; C_REMAINDER];
        let mut abuf = [0i64; C_INTERMEDIATE + 1];
        let odd = (self.exponent & 1) != 0;

        // a points into abuf; if even exponent we skip abuf[0]
        let a_offset: usize = if !odd { 1 } else { 0 };

        for i in 0..self.length as usize {
            r[2 * i] = (self.d[i] >> (C_LOG / 2)) as i64;
            r[2 * i + 1] = (self.d[i] as i64) & ((1i64 << (C_LOG / 2)) - 1);
        }
        // rest already zero

        // a[] = &abuf[a_offset], so a[i] = abuf[a_offset + i]
        // initialize a[0..C_INTERMEDIATE] = 0
        for i in 0..C_INTERMEDIATE {
            abuf[a_offset + i] = 0;
        }

        // first two half-digits via integer arithmetic
        let first: u64 = ((r[0] as u64) << (3 * C_LOG / 2 as u32))
            | ((r[1] as u64) << (2 * C_LOG / 2 as u32))
            | ((r[2] as u64) << (1 * C_LOG / 2 as u32))
            | (r[3] as u64);
        r[0] = 0;
        r[1] = 0;
        r[2] = 0;
        r[3] = 0;
        let mut root: u64 = 1u64 << C_LOG;
        loop {
            let old_root = root;
            root = (root + first / root) / 2;
            if old_root == root || old_root == root + 1 {
                break;
            }
        }
        abuf[a_offset] = (root >> (C_LOG / 2)) as i64;
        assert_bf!(abuf[a_offset] >= 1);
        assert_bf!(abuf[a_offset] < (1i64 << (C_LOG / 2)));
        abuf[a_offset + 1] = (root & ((1u64 << (C_LOG / 2)) - 1)) as i64;

        let mut remainder = first as i64 - (root * root) as i64;
        assert_bf!(remainder < (2i64 << C_LOG));
        assert_bf!(remainder > -(2i64 << C_LOG));
        let mut denominator = -2i64 * root as i64;

        for i in 2..C_INTERMEDIATE {
            remainder <<= C_LOG / 2;
            remainder += r[i + 2];
            r[i + 2] = 0;

            let delta = remainder / denominator;
            remainder -= denominator * delta;
            assert_bf!(remainder < (2i64 << C_LOG));
            assert_bf!(remainder > -(2i64 << C_LOG));
            assert_bf!(r[i] == 0);

            let j_limit = if 2 * i + 1 < C_INTERMEDIATE {
                let square = delta * delta;
                r[2 * i + 1] -= square;
                i
            } else {
                C_INTERMEDIATE - i
            };

            if j_limit > 2 {
                // C++ loop: while (j-- > 3) {...} then r[i+j+1] += 2*a[j]*delta
                // After the C++ loop, j==3.
                let mut j = j_limit;
                while j > 3 {
                    j -= 1;
                    let p = r[i + j + 1] + 2 * abuf[a_offset + j] * delta;
                    r[i + j + 1] = p & ((1i64 << (C_LOG / 2)) - 1);
                    r[i + j] += p >> (C_LOG / 2);
                }
                // In C++, j==2 after the while loop (post-decrement exits with j=2)
                r[i + 3] += 2 * abuf[a_offset + 2] * delta;
            }

            abuf[a_offset + i] = -delta;
        }

        // carries
        let mut i = C_INTERMEDIATE;
        while i > 1 {
            i -= 1;
            let top = abuf[a_offset + i] >> (C_LOG / 2 as u32) as i64;
            abuf[a_offset + i - 1] += top;
            abuf[a_offset + i] -= top << (C_LOG / 2);
        }

        // combine half-digits
        self.exponent >>= 1;
        self.length = C_DIGITS as u16;
        let previous_digit =
            (abuf[a_offset + 2 * C_DIGITS] << (C_LOG / 2)) + abuf[a_offset + 2 * C_DIGITS + 1];

        let x0 = (abuf[0] << (C_LOG / 2)) + abuf[1];
        assert_bf!(x0 >= 1);
        assert_bf!(x0 < (1i64 << C_LOG));
        self.d[0] = x0 as u32;
        for i in 1..C_DIGITS {
            let x = (abuf[2 * i] << (C_LOG / 2)) + abuf[2 * i + 1];
            assert_bf!(x >= 0);
            assert_bf!(x < (1i64 << C_LOG));
            self.d[i] = x as u32;
        }
        self.round_signed(false, previous_digit)
    }

    // -----------------------------------------------------------------------
    // Trig / transcendental
    // -----------------------------------------------------------------------

    fn quadrant(&mut self) -> i64 {
        let c = cache();
        let mut div2pi = self.clone();
        div2pi.mult(&c.over_two_pi);
        let mut extra = div2pi.clone();
        extra.trunc();
        div2pi.sub(&extra);
        extra.mult(&c.two_pi);
        self.sub(&extra);

        div2pi.mult_int(8, 0);
        div2pi.trunc();
        let quadrant = div2pi.to_integer();

        match quadrant {
            7 => {
                let two_pi = c.two_pi.clone();
                self.sub(&two_pi);
            }
            6 | 5 => {
                let tpot = c.three_pi_over_two.clone();
                self.sub(&tpot);
            }
            4 | 3 => {
                let pi = c.pi.clone();
                self.sub(&pi);
            }
            2 | 1 => {
                let pot = c.pi_over_two.clone();
                self.sub(&pot);
            }
            0 => {
                if self.is_negative {
                    // quadrant becomes 7 effectively... but we return 0+7?
                    // Actually C++ code does quadrant += 7 only for case 0 if _isNegative
                    // return quadrant (which is 0 + 7 = 7)... but it doesn't do that
                    // Looking again: case 0: if (_isNegative) quadrant += 7; break;
                    // So we fall through and return quadrant which is 0 or 7.
                    return 0 + 7;
                }
            }
            -1 | -2 => {
                let pot = c.pi_over_two.clone();
                self.add(&pot);
                return quadrant + 7;
            }
            -3 | -4 => {
                let pi = c.pi.clone();
                self.add(&pi);
                return quadrant + 7;
            }
            -5 | -6 => {
                let tpot = c.three_pi_over_two.clone();
                self.add(&tpot);
                return quadrant + 7;
            }
            -7 => {
                let two_pi = c.two_pi.clone();
                self.add(&two_pi);
                return quadrant + 7;
            }
            _ => panic!("bad quadrant {}", quadrant),
        }

        let mut limit = BigFloat::from_int_exp(3217, 0);
        limit.div_int(4096, 0);
        assert_bf!(self.compare_absolute(&limit) <= 0);
        quadrant
    }

    fn partial_sin(&mut self) -> &mut Self {
        let c = cache();
        let mut x2 = self.clone();
        x2.mult(self as &BigFloat);
        let mut x4 = x2.clone();
        x4.mult(&x2.clone());
        let mut x6 = x4.clone();
        x6.mult(&x2.clone());
        let mut x8 = x6.clone();
        x8.mult(&x2.clone());

        let mut sin = BigFloat::from_int(0);
        let mut old_sin = BigFloat::from_int(0);
        let mut power = self.clone();
        let mut i: i64 = 7;
        let over_fact_len = c.over_fact.len() as i64;
        assert_bf!(over_fact_len < (1i64 << (63 / 6)));
        while i < over_fact_len {
            let ii = i;
            let mut sum = BigFloat::from_int(ii * (ii - 1) * (ii - 2) * (ii - 3) * (ii - 4) * (ii - 5));
            let mut term = x2.clone();
            term.mult_int(ii * (ii - 1) * (ii - 2) * (ii - 3), 0);
            sum.sub(&term);
            term.copy_from(&x4);
            term.mult_int(ii * (ii - 1), 0);
            sum.add(&term);
            sum.sub(&x6);

            sum.mult(&c.over_fact[ii as usize]);
            sum.mult(&power);
            sin.add(&sum);

            if sin.compare(&old_sin) == 0 {
                break;
            }
            old_sin.copy_from(&sin);
            power.mult(&x8.clone());
            i += 8;
        }
        assert_bf!(i < over_fact_len);
        self.copy_from(&sin)
    }

    fn partial_cos(&mut self) -> &mut Self {
        let c = cache();
        let mut x2 = self.clone();
        x2.mult(self as &BigFloat);
        let mut x4 = x2.clone();
        x4.mult(&x2.clone());
        let mut x6 = x4.clone();
        x6.mult(&x2.clone());
        let mut x8 = x6.clone();
        x8.mult(&x2.clone());

        let mut cos = BigFloat::from_int(0);
        let mut old_cos = BigFloat::from_int(0);
        let mut power = BigFloat::from_int(1);
        let mut i: i64 = 6;
        let over_fact_len = c.over_fact.len() as i64;
        assert_bf!(over_fact_len < (1i64 << (63 / 6)));
        while i < over_fact_len {
            let ii = i;
            let mut sum = BigFloat::from_int(ii * (ii - 1) * (ii - 2) * (ii - 3) * (ii - 4) * (ii - 5));
            let mut term = x2.clone();
            term.mult_int(ii * (ii - 1) * (ii - 2) * (ii - 3), 0);
            sum.sub(&term);
            term.copy_from(&x4);
            term.mult_int(ii * (ii - 1), 0);
            sum.add(&term);
            sum.sub(&x6);

            sum.mult(&c.over_fact[ii as usize]);
            sum.mult(&power);
            cos.add(&sum);

            if cos.compare(&old_cos) == 0 {
                break;
            }
            old_cos.copy_from(&cos);
            power.mult(&x8.clone());
            i += 8;
        }
        assert_bf!(i < over_fact_len);
        self.copy_from(&cos)
    }

    pub fn sin(&mut self) -> &mut Self {
        if self.is_special() {
            if self.is_zero() {
                return self.copy_from(&BigFloat::from_int(0));
            } else {
                return self.nan();
            }
        }
        let quadrant = self.quadrant();
        match quadrant {
            7 | 0 => { self.partial_sin(); }
            1 | 2 => { self.partial_cos(); }
            3 | 4 => { self.partial_sin(); self.negate(); }
            5 | 6 => { self.partial_cos(); self.negate(); }
            _ => {}
        }
        self
    }

    pub fn csc(&mut self) -> &mut Self {
        self.sin();
        self.invert()
    }

    pub fn cos(&mut self) -> &mut Self {
        if self.is_special() {
            if self.is_zero() {
                let one = BigFloat::from_int(1);
                return self.copy_from(&one);
            } else {
                return self.nan();
            }
        }
        let quadrant = self.quadrant();
        match quadrant {
            7 | 0 => { self.partial_cos(); }
            1 | 2 => { self.partial_sin(); self.negate(); }
            3 | 4 => { self.partial_cos(); self.negate(); }
            5 | 6 => { self.partial_sin(); }
            _ => {}
        }
        self
    }

    pub fn sec(&mut self) -> &mut Self {
        self.cos();
        self.invert()
    }

    pub fn tan(&mut self) -> &mut Self {
        if self.is_special() {
            if self.is_zero() {
                let z = BigFloat::from_int(0);
                return self.copy_from(&z);
            } else {
                return self.nan();
            }
        }
        let quadrant = self.quadrant();
        match quadrant {
            7 | 0 | 3 | 4 => {
                // sine is more accurate, derive cosine
                self.partial_sin();
                let mut sin2 = self.clone();
                sin2.mult(&self.clone());
                let mut cos = BigFloat::from_int(1);
                cos.sub(&sin2);
                cos.sqrt();
                self.div(&cos);
            }
            2 | 6 => {
                // cosine more accurate, negate result
                self.partial_cos();
                let mut cos2 = self.clone();
                cos2.mult(&self.clone());
                let mut sin = BigFloat::from_int(1);
                sin.sub(&cos2);
                sin.sqrt();
                sin.is_negative = !sin.is_negative;
                self.div(&sin);
            }
            1 | 5 => {
                self.partial_cos();
                let mut cos2 = self.clone();
                cos2.mult(&self.clone());
                let mut sin = BigFloat::from_int(1);
                sin.sub(&cos2);
                sin.sqrt();
                self.div(&sin);
            }
            _ => {}
        }
        self
    }

    pub fn exp(&mut self) -> &mut Self {
        let c = cache();
        if self.is_special() {
            if self.is_nan() {
                return self;
            } else if self.is_zero() {
                let one = BigFloat::from_int(1);
                return self.copy_from(&one);
            } else if self.is_p_inf() {
                return self;
            } else if self.is_n_inf() {
                return self.zero(false);
            }
        }

        // only calculate e on [-1/16, 1/16]
        let scale = 1i64 << (-E_POWER_NEG as u32 - 1);
        let mut whole = self.clone();
        whole.mult_int(scale, 0);
        whole.trunc();
        whole.div_int(scale, 0);
        self.sub(&whole);

        let mut x2 = self.clone();
        x2.mult(self as &BigFloat);
        let mut x4 = x2.clone();
        x4.mult(&x2.clone());
        let mut x8 = x4.clone();
        x8.mult(&x4.clone());

        let mut exp_val = BigFloat::from_int(1);
        let mut old_exp = BigFloat::from_int(1);
        let mut power = self.clone();
        let over_fact_len = c.over_fact.len() as i64;
        let mut i: i64 = 8;
        while i < over_fact_len {
            let ii = i;
            let mut even = BigFloat::from_int(ii * (ii - 1) * (ii - 2) * (ii - 3) * (ii - 4));
            let mut term = BigFloat::from_int(ii);
            term.mult(&x4);
            even.add(&term);
            even.mult(&x2);
            even.add_int(ii * (ii - 1) * (ii - 2) * (ii - 3) * (ii - 4) * (ii - 5) * (ii - 6), 0);
            term.copy_from(&BigFloat::from_int(ii * (ii - 1) * (ii - 2)));
            term.mult(&x4);
            even.add(&term);

            let mut odd = BigFloat::from_int(ii * (ii - 1) * (ii - 2) * (ii - 3));
            term.copy_from(&BigFloat::from_int(1));
            term.mult(&x4);
            odd.add(&term);
            odd.mult(&x2);
            odd.add_int(ii * (ii - 1) * (ii - 2) * (ii - 3) * (ii - 4) * (ii - 5), 0);
            term.copy_from(&BigFloat::from_int(ii * (ii - 1)));
            term.mult(&x4);
            odd.add(&term);

            let self_clone = self.clone();
            odd.mult(&self_clone);
            odd.add(&even);
            odd.mult(&c.over_fact[ii as usize]);
            odd.mult(&power);

            exp_val.add(&odd);

            if exp_val.compare(&old_exp) == 0 {
                break;
            }
            old_exp.copy_from(&exp_val);
            power.mult(&x8.clone());
            i += 8;
        }

        // handle whole number multiple of 1/scale
        if !whole.is_negative {
            for i in 0..whole.length as usize {
                for j in 0..C_LOG as i64 {
                    if (whole.d[i] & (1u32 << j)) != 0 {
                        let index = C_LOG as i64 * (whole.exponent - i as i64) + j;
                        let e_power_len = c.e_power_len;
                        if index < e_power_len {
                            assert_bf!(index > E_POWER_NEG);
                            let ep = c.e_power[(index - E_POWER_NEG) as usize].clone();
                            exp_val.mult(&ep);
                        } else {
                            exp_val.inf(false);
                        }
                    }
                }
            }
        } else if whole.is_negative {
            for i in 0..whole.length as usize {
                for j in 0..C_LOG as i64 {
                    if (whole.d[i] & (1u32 << j)) != 0 {
                        let index = C_LOG as i64 * (whole.exponent - i as i64) + j;
                        let e_power_len = c.e_power_len;
                        if index < e_power_len {
                            assert_bf!(index > E_POWER_NEG);
                            let ep = c.e_inv_power[(index - E_POWER_NEG) as usize].clone();
                            exp_val.mult(&ep);
                        } else {
                            exp_val.zero(false);
                        }
                    }
                }
            }
        }

        self.copy_from(&exp_val)
    }

    pub fn a_sin(&mut self) -> &mut Self {
        let mut denom = self.clone();
        denom.mult(&self.clone());
        denom.sub_int(1, 0);
        denom.negate();
        denom.sqrt();
        denom.add_int(1, 0);
        self.div(&denom);
        self.a_tan();
        self.mult_int(2, 0)
    }

    pub fn a_cos(&mut self) -> &mut Self {
        let mut num = self.clone();
        num.mult(&self.clone());
        num.sub_int(1, 0);
        num.negate();
        num.sqrt();
        let mut denom = self.clone();
        denom.add_int(1, 0);
        num.div(&denom);
        num.a_tan();
        num.mult_int(2, 0);
        self.copy_from(&num)
    }

    pub fn a_tan(&mut self) -> &mut Self {
        if self.is_nan() {
            return self;
        }
        let c = cache();

        let flip = self.compare_absolute(&BigFloat::from_int(1)) > 0;
        if flip {
            self.invert();
        }

        const C_ITER: i32 = 2;
        for _ in 0..C_ITER {
            let mut denom = self.clone();
            denom.mult(&self.clone());
            denom.add_int(1, 0);
            denom.sqrt();
            denom.add_int(1, 0);
            self.div(&denom);
        }

        let mut x2 = self.clone();
        x2.mult(self as &BigFloat);
        let mut x4 = x2.clone();
        x4.mult(&x2.clone());

        let mut a_tan = BigFloat::from_int(0);
        let mut old_a_tan = BigFloat::from_int(0);
        let mut power = self.clone();
        let mut i: i64 = 1;
        loop {
            let ii = i;
            let mut term = BigFloat::from_int(ii);
            term.mult(&x2);
            term.sub_int(ii + 2, 0);
            term.mult(&power);
            term.div_int(ii * (ii + 2), 0);
            a_tan.sub(&term);

            if a_tan.compare(&old_a_tan) == 0 {
                break;
            }
            old_a_tan.copy_from(&a_tan);
            power.mult(&x4.clone());
            i += 4;
        }
        self.copy_from(&a_tan);

        self.mult_int(1i64 << C_ITER, 0);
        if flip {
            self.negate();
            if self.is_negative {
                let pot = c.pi_over_two.clone();
                self.sub(&pot);
            } else {
                let pot = c.pi_over_two.clone();
                self.add(&pot);
            }
        }

        self
    }

    pub fn ln(&mut self) -> &mut Self {
        let c = cache();
        if self.is_negative {
            return self.nan();
        } else if self.is_special() {
            if self.is_zero() {
                return self.n_inf();
            } else if self.is_inf() {
                return self.inf(false);
            } else {
                return self.nan();
            }
        }

        // get remainder in [1-delta, delta] for small delta
        let mut whole = BigFloat::new();
        let min_digit: i64 = E_POWER_NEG / C_LOG as i64 - 2;
        let offset: i64 = min_digit * C_LOG as i64;

        let e_power_neg = E_POWER_NEG;
        let e_power_len = c.e_power_len;

        if self.exponent >= 0 {
            let mut i = e_power_neg + 1;
            while i < e_power_len && self.compare(&c.e_power[(i - e_power_neg) as usize]) >= 0 {
                i += 1;
            }
            whole.exponent = min_digit + (i - 1 - offset) / C_LOG as i64;
            let len = whole.exponent - min_digit;
            whole.length = (len.min(C_DIGITS as i64).max(0)) as u16;
            for j in 0..whole.length as usize {
                whole.d[j] = 0;
            }
            while i > e_power_neg + 1 {
                i -= 1;
                if self.compare(&c.e_power[(i - e_power_neg) as usize]) >= 0 {
                    let ep_inv = c.e_inv_power[(i - e_power_neg) as usize].clone();
                    self.mult(&ep_inv);
                    let bit_pos = whole.exponent - (i - offset) / C_LOG as i64 - min_digit;
                    whole.d[bit_pos as usize] += 1u32 << ((i + offset) % C_LOG as i64) as u32;
                }
            }
        } else {
            whole.is_negative = true;
            let mut i = e_power_neg + 1;
            while i < e_power_len && self.compare(&c.e_inv_power[(i - e_power_neg) as usize]) <= 0 {
                i += 1;
            }
            whole.exponent = min_digit + (i - 1 - offset) / C_LOG as i64;
            let len = whole.exponent - min_digit;
            assert_bf!(len >= 0);
            whole.length = (len.min(C_DIGITS as i64)) as u16;
            for j in 0..whole.length as usize {
                whole.d[j] = 0;
            }
            while i > e_power_neg + 1 {
                i -= 1;
                if self.compare(&c.e_inv_power[(i - e_power_neg) as usize]) <= 0 {
                    let ep = c.e_power[(i - e_power_neg) as usize].clone();
                    self.mult(&ep);
                    let bit_pos = whole.exponent - (i - offset) / C_LOG as i64 - min_digit;
                    whole.d[bit_pos as usize] += 1u32 << ((i + offset) % C_LOG as i64) as u32;
                }
            }
        }

        let mut x = self.clone();
        x.sub_int(1, 0);
        let mut x2 = x.clone();
        x2.mult(&x.clone());
        let mut power = x.clone();
        let mut old_ln = BigFloat::from_int(1);
        let mut ln_val = BigFloat::from_int(0);
        let mut i: i64 = 1;
        loop {
            let ii = i;
            let mut term = BigFloat::from_int(ii);
            term.mult(&x);
            term.sub_int(ii + 1, 0);
            term.div_int(ii * (ii + 1), 0);
            term.mult(&power);
            ln_val.sub(&term);
            if old_ln.compare(&ln_val) == 0 {
                break;
            }
            old_ln.copy_from(&ln_val);
            power.mult(&x2.clone());
            i += 2;
        }

        ln_val.add(&whole);
        self.copy_from(&ln_val)
    }

    pub fn power(&mut self, n: &BigFloat) -> &mut Self {
        self.ln();
        self.mult(n);
        self.exp()
    }

    pub fn power_int(&mut self, n: i64, exponent: i64) -> &mut Self {
        panic!("not implemented");
    }

    pub fn log(&mut self, n: &BigFloat) -> &mut Self {
        let mut lnn = n.clone();
        lnn.ln();
        self.ln();
        self.invert();
        self.mult(&lnn)
    }

    pub fn rand(&mut self) -> &mut Self {
        panic!("not implemented")
    }

    pub fn rand_norm(&mut self) -> &mut Self {
        panic!("not implemented")
    }

    // -----------------------------------------------------------------------
    // Constants
    // -----------------------------------------------------------------------

    pub fn pi() -> &'static BigFloat {
        &cache().pi
    }

    pub fn e_const() -> &'static BigFloat {
        &cache().e
    }

    pub fn const_zero() -> &'static BigFloat {
        &cache().zero
    }

    pub fn const_one() -> BigFloat {
        BigFloat::from_int(1)
    }

    pub fn const_minus_one() -> BigFloat {
        BigFloat::from_int(-1)
    }

    // -----------------------------------------------------------------------
    // Gaussian elimination
    // -----------------------------------------------------------------------

    pub fn gaussian_elimination(m: &mut Vec<Vec<BigFloat>>, rows: usize, cols: usize) {
        // build triangular matrix
        for i in 0..cols {
            // swap a row to i such that m[i][i] is nonzero
            let mut k = i;
            while k < rows {
                if !m[k][i].is_zero() {
                    break;
                }
                k += 1;
            }
            assert_bf!(k < rows);
            if k != i {
                m.swap(i, k);
            }

            // do the elimination
            for k in (i + 1)..rows {
                let mut c = m[k][i].clone();
                {
                    let mii = m[i][i].clone();
                    c.div(&mii);
                }
                for j in i..(cols + 1) {
                    let temp_val = {
                        let mut t = m[i][j].clone();
                        t.mult(&c);
                        t
                    };
                    m[k][j].sub(&temp_val);
                }
            }
        }

        // use triangular matrix to find results
        let mut i = cols;
        while i > 0 {
            i -= 1;
            for j in (i + 1)..cols {
                let temp_val = {
                    let mut t = m[i][j].clone();
                    let mjcols = m[j][cols].clone();
                    t.mult(&mjcols);
                    t
                };
                m[i][cols].sub(&temp_val);
            }
            let mii = m[i][i].clone();
            m[i][cols].div(&mii);
        }
    }

    // -----------------------------------------------------------------------
    // Round integer helper
    // -----------------------------------------------------------------------

    pub fn round_integer(value: i64) -> i64 {
        let negative = value < 0;
        let mut x: u64 = if negative { (-(value as i128)) as u64 } else { value as u64 };

        let mut exp: u32 = 0;
        while exp < 64 {
            if x < (C_RANGE << exp) {
                break;
            }
            exp += C_LOG;
        }

        let precision = C_LOG * (C_DIGITS as u32 - 1);
        if exp > precision {
            x += 1u64 << (exp - precision - 1);
            x >>= exp - precision;
            x <<= exp - precision;
        }

        if negative { -(x as i64) } else { x as i64 }
    }

    // -----------------------------------------------------------------------
    // Printing
    // -----------------------------------------------------------------------

    pub fn print(&self) {
        print!("exp={}, {} ", self.exponent, if self.is_negative { '-' } else { '+' });
        for i in 0..self.length as usize {
            print!("{:x} ", self.d[i]);
        }
        println!();
    }

    pub fn print_hex(&self) {
        if self.is_negative {
            print!("-");
        }
        if self.is_special() {
            if self.is_zero() {
                print!("0");
            } else if self.is_inf() {
                print!("inf");
            } else if self.is_nan() {
                print!("NaN");
            }
        } else if self.exponent + 1 > self.length as i64 || self.exponent < 0 {
            print!("0x");
            for i in 0..self.length as usize {
                if i == 0 {
                    print!("{:x}", self.d[0]);
                } else {
                    if i == 1 {
                        print!(".");
                    }
                    print!("{:08x}", self.d[i]);
                }
            }
            print!(":e{}", self.exponent);
        } else {
            print!("0x");
            for i in 0..self.length as usize {
                if i == 0 {
                    print!("{:x}", self.d[0]);
                } else {
                    if i as i64 == self.exponent + 1 {
                        print!(".");
                    }
                    print!("{:08x}", self.d[i]);
                }
            }
        }
    }

    pub fn print_decimal(&self) {
        if self.is_negative {
            print!("-");
        }
        if self.is_special() {
            if self.is_zero() {
                print!("0");
            } else if self.is_inf() {
                print!("inf");
            } else if self.is_nan() {
                print!("NaN");
            }
        } else if self.exponent + 1 > self.length as i64 || self.exponent < 0 {
            print!(" dunno ");
        } else {
            let mut top = self.clone();
            top.is_negative = false;

            let mut buf: Vec<u8> = Vec::new();
            while !top.is_zero() {
                let mut x = top.clone();
                x.div_int(10, 0);
                x.trunc();
                x.mult_int(10, 0);
                x.sub(&top);
                buf.push(b'0'.wrapping_sub(x.to_integer() as u8));
                top.add(&x);
                top.div_int(10, 0);
            }
            buf.reverse();
            print!("{}", std::str::from_utf8(&buf).unwrap_or("?"));
        }
    }

    pub fn print_double(&self) {
        print!("{:.19}", self.to_double());
    }

    pub fn print_continued_fraction(&self) {
        if self.is_zero() {
            print!("0.0");
            return;
        }

        let mut n1 = self.clone();
        let mut x: i64 = 0;
        let mut sign = "";
        if n1.is_negative() {
            sign = "-";
            n1.negate();
        }

        while n1.compare_int(2, 0) >= 0 {
            n1.div_int(2, 0);
            x += 1;
        }
        while n1.compare_int(1, 0) < 0 {
            n1.mult_int(2, 0);
            x -= 1;
        }

        if x < -400 {
            print!("0.0");
            return;
        }

        print!("(");

        let mut r = n1.clone();
        n1.trunc();
        let mut d1 = BigFloat::from_int(1);

        r.sub(&n1);
        if r.is_zero() {
            print!("0x{:x})", n1.to_integer());
            return;
        }
        r.invert();
        let mut d2 = r.clone();
        d2.trunc();
        r.sub(&d2);
        let mut n2 = n1.clone();
        n2.mult(&d2);
        n2.add_int(1, 0);

        loop {
            r.invert();
            let mut c = r.clone();
            c.trunc();
            r.sub(&c);

            let mut temp = n2.clone();
            temp.mult(&c);
            n1.add(&temp);

            temp.copy_from(&d2);
            temp.mult(&c);
            d1.add(&temp);

            if r.is_zero() || n1.exponent * C_LOG as i64 >= 32 {
                print!(
                    "{}static_cast<double>({}) / {}",
                    sign,
                    n2.to_integer(),
                    d2.to_integer()
                );
                break;
            }

            r.invert();
            c.copy_from(&r);
            c.trunc();
            r.sub(&c);

            temp.copy_from(&n1);
            temp.mult(&c);
            n2.add(&temp);

            temp.copy_from(&d1);
            temp.mult(&c);
            d2.add(&temp);

            if r.is_zero() || n2.exponent * C_LOG as i64 >= 32 {
                print!(
                    "{}static_cast<double>({}) / {}",
                    sign,
                    n1.to_integer(),
                    d1.to_integer()
                );
                break;
            }
        }

        if x > 0 && x < 32 {
            print!(" * 0x{:x}", 1u64 << x);
        } else if x < 0 && x > -32 {
            print!(" / 0x{:x}", 1u64 << (-x));
        } else {
            print!(" * PowerOfTwo({})", x);
        }

        print!(")");
    }

    pub fn to_fraction(&self, num: &mut BigFloat, denom: &mut BigFloat, iter: i32) {
        let mut remainder = self.clone();
        remainder.is_negative = false;
        num.copy_from(self);
        num.trunc();
        remainder.sub(num as &BigFloat);
        if remainder.is_zero()
            || remainder.exponent < -(C_DIGITS as i64 / 2)
            || iter == 0
        {
            denom.from_integer(1, 0);
        } else {
            remainder.invert();
            let mut sub_denom = BigFloat::new();
            remainder.to_fraction(denom, &mut sub_denom, iter - 1);
            num.mult(denom as &BigFloat);
            num.add(&sub_denom);
        }
        num.is_negative = self.is_negative;
    }

    // -----------------------------------------------------------------------
    // Unit test helpers (feature-gated)
    // -----------------------------------------------------------------------

    #[cfg(feature = "bigfloat-test")]
    fn test_integer(n: &BigFloat, value: i64) {
        let x = BigFloat::round_integer(value);
        assert_bf!(n.to_integer() == x, "n={}, x={}", n.to_integer() as i32, x as i32);
    }

    #[cfg(feature = "bigfloat-test")]
    fn test_add(x: i64, y: i64) {
        let bx = BigFloat::from_int(x);
        let by = BigFloat::from_int(y);
        let mut bz = bx.clone();
        BigFloat::test_integer(&bz.clone().add(&by).clone(), x + y);
        bz.copy_from(&bx);
        BigFloat::test_integer(&bz.clone().sub(&by).clone(), x - y);
    }

    #[cfg(feature = "bigfloat-test")]
    fn test_mult(x: i64, ex: i64, y: i64, ey: i64) {
        let px = if x < 0 { -x } else { x };
        let py = if y < 0 { -y } else { y };
        assert_bf!(px < (1i64 << 31));
        assert_bf!(py < (1i64 << 31));
        let pz = BigFloat::round_integer(px * py);

        let bx = BigFloat::from_int_exp(x, ex);
        let by = BigFloat::from_int_exp(y, ey);
        let mut bz = bx.clone();

        if pz == 0 {
            bz.mult(&by);
            assert_bf!(bz.to_integer() == 0);
            return;
        }

        // Use u64 for pz2 to match C++ unsigned comparison behavior
        let mut pz2 = pz as u64;
        let mut count = 0i64;
        while pz2 < (1u64 << (64 - C_LOG as u32)) && count < 64 {
            pz2 <<= C_LOG;
            count += 1;
        }
        let exponent = ex + ey + (64 / C_LOG as i64 - count - 1);

        if exponent > C_MAX_EXPONENT {
            return;
        }

        bz.mult(&by);
        if exponent < C_MIN_EXPONENT {
            assert_bf!(bz.compare_absolute(&BigFloat::from_int(0)) == 0);
        } else {
            assert_bf!(bz.to_digits() == pz2 as u64);
            assert_bf!(bz.to_exponent() == exponent);
            assert_bf!(bz.is_negative() == ((x < 0) != (y < 0)));
        }
    }

    #[cfg(feature = "bigfloat-test")]
    fn test_inverse(x: i64, ex: i64) {
        let bx = BigFloat::from_int_exp(x, ex);
        let mut bi = bx.clone();
        bi.invert();
        let mut bq = bi.clone();
        bq.mult(&bx);
        if bx.to_exponent() >= -C_MAX_EXPONENT && bi.is_zero() {
            return;
        }
        bq.sub_int(1, 0);
        assert_bf!(
            bq.compare_int(0, 0) == 0 || bq.to_exponent() <= 1 - C_DIGITS as i64
        );
    }

    #[cfg(feature = "bigfloat-test")]
    fn test_sqrt(x: i64, ex: i64) {
        let bx = BigFloat::from_int_exp(x, ex);
        let mut br = bx.clone();
        br.sqrt();

        let mut bd = br.clone();
        bd.mult(&br);
        bd.sub(&bx);
        let mut bo = bx.clone();
        bo.invert();
        bo.mult(&bd);
        assert_bf!(
            bo.compare_int(0, 0) == 0 || bo.to_exponent() <= 1 - C_DIGITS as i64
        );

        let mut bd2 = bx.clone();
        bd2.div(&br);
        bd2.sub(&br);
        let mut bo2 = br.clone();
        bo2.invert();
        bo2.mult(&bd2);
        assert_bf!(
            bo2.compare_int(0, 0) == 0 || bo2.to_exponent() <= 1 - C_DIGITS as i64
        );
    }

    #[cfg(feature = "bigfloat-test")]
    pub fn unit_test() {
        let mut b = BigFloat::new();
        let zero = BigFloat::new();
        assert_bf!(b.compare(&zero) == 0);

        b.from_integer(1, 0);
        assert_bf!(b.compare(&zero) == 1);
        assert_bf!(b.compare(&b.clone()) == 0);

        b.from_integer(-1, 0);
        assert_bf!(b.compare(&zero) == -1);
        assert_bf!(b.compare(&b.clone()) == 0);

        let mut c = BigFloat::new();
        c.from_integer(-(C_RANGE as i64), 0);
        BigFloat::test_integer(&c, -(C_RANGE as i64));

        BigFloat::test_sqrt(2, 0);

        let c_max_int: u64 = 1u64 << (C_LOG as u64 * C_DIGITS as u64);

        // check if inverse works
        let mut ex = C_MIN_EXPONENT - C_DIGITS as i64 + 1;
        while ex <= C_MAX_EXPONENT - C_DIGITS as i64 + 1 {
            let mut x = 1i64 << (C_LOG * (C_DIGITS as u32 - 1));
            while (x as u64) < c_max_int {
                BigFloat::test_inverse(x, ex);
                BigFloat::test_inverse(-x, ex);
                x += 1;
            }
            ex += 1;
        }
        println!("Inverse works");

        // check if square root works
        let mut ex = C_MIN_EXPONENT - C_DIGITS as i64 + 1;
        while ex < C_MAX_EXPONENT - C_DIGITS as i64 + 1 {
            let mut x = 1i64 << (C_LOG * (C_DIGITS as u32 - 1));
            while (x as u64) < c_max_int {
                BigFloat::test_sqrt(x, ex);
                x += 1;
            }
            ex += 1;
        }
        println!("Square root works");

        // check if addition and subtraction work
        let mut coef: u32 = 1;
        for _q in 0..2 * C_DIGITS {
            let mut x: i64 = 0;
            while (x as u64) < coef as u64 * c_max_int {
                let mut y: i64 = 0;
                while (y as u64) < c_max_int {
                    BigFloat::test_add(x, y);
                    BigFloat::test_add(x, -y);
                    BigFloat::test_add(-x, y);
                    BigFloat::test_add(-x, -y);
                    y += 1;
                }
                x += coef as i64;
            }
            coef = coef.wrapping_mul(C_RANGE as u32);
        }
        println!("Addition and Subtraction work");

        // check if multiplication works
        BigFloat::test_mult(0, 0, 1, 0);
        BigFloat::test_mult(-1, 0, 0, 0);
        let mut ex = C_MIN_EXPONENT - C_DIGITS as i64 + 1;
        while ex <= C_MAX_EXPONENT - C_DIGITS as i64 + 1 {
            let mut x = 1i64 << (C_LOG * (C_DIGITS as u32 - 1));
            while (x as u64) < c_max_int {
                let mut ey = C_MIN_EXPONENT - C_DIGITS as i64 + 1;
                while ey <= C_MAX_EXPONENT - C_DIGITS as i64 + 1 {
                    let mut y = 1i64 << (C_LOG * (C_DIGITS as u32 - 1));
                    while (y as u64) < c_max_int {
                        BigFloat::test_mult(x, ex, y, ey);
                        BigFloat::test_mult(x, ex, -y, ey);
                        BigFloat::test_mult(-x, ex, y, ey);
                        BigFloat::test_mult(-x, ex, -y, ey);
                        y += 1;
                    }
                    ey += 1;
                }
                x += 1;
            }
            ex += 1;
        }
        println!("Multiplication works");
    }
}

// -----------------------------------------------------------------------
// Cache
// -----------------------------------------------------------------------

struct Cache {
    e: BigFloat,
    zero: BigFloat,
    e_power: Vec<BigFloat>,     // index as (i - E_POWER_NEG) where i in E_POWER_NEG+1..e_power_len
    e_inv_power: Vec<BigFloat>, // same indexing
    e_power_len: i64,
    pi: BigFloat,
    two_pi: BigFloat,
    over_two_pi: BigFloat,
    pi_over_two: BigFloat,
    three_pi_over_two: BigFloat,
    pi_over_four: BigFloat,
    over_fact: Vec<BigFloat>,
}

static CACHE: std::sync::OnceLock<Cache> = std::sync::OnceLock::new();

fn cache() -> &'static Cache {
    CACHE.get_or_init(Cache::init)
}

impl Cache {
    fn init() -> Cache {
        let c_max_iter = 1000usize;

        // fill pi using Brent-Salamin algorithm
        let mut a = BigFloat::from_int(1);
        let mut b = BigFloat::from_int(2);
        b.sqrt();
        b.invert();
        let mut t = BigFloat::from_int(4);
        t.invert();
        let mut p = BigFloat::from_int(1);
        let mut old_pi = BigFloat::from_int(3);
        let mut new_pi = BigFloat::new();

        let mut i = 0usize;
        while i < c_max_iter {
            let mut a2 = a.clone();
            a2.add(&b);
            a2.div_int(2, 0);

            b.mult(&a);
            b.sqrt();

            let mut t2 = a.clone();
            t2.sub(&a2);
            t2.mult(&t2.clone());
            t2.mult(&p);
            t.sub(&t2);

            p.mult_int(2, 0);
            a.copy_from(&a2);

            new_pi.copy_from(&a);
            new_pi.add(&b);
            new_pi.mult(&new_pi.clone());
            new_pi.div(&t);
            new_pi.div_int(4, 0);

            if new_pi.compare(&old_pi) == 0 {
                break;
            }
            old_pi.copy_from(&new_pi);
            i += 1;
        }
        assert_bf!(i < c_max_iter);

        // now compute 2pi, pi, pi/2, pi/4 using same numerator
        old_pi.copy_from(&a);
        old_pi.add(&b);
        old_pi.mult(&old_pi.clone());
        let mut pi_divisor = t.clone();

        pi_divisor.mult_int(2, 0);
        new_pi.copy_from(&old_pi);
        new_pi.div(&pi_divisor);
        let two_pi = new_pi.clone();

        pi_divisor.mult_int(2, 0);
        new_pi.copy_from(&old_pi);
        new_pi.div(&pi_divisor);
        let pi = new_pi.clone();

        pi_divisor.mult_int(2, 0);
        new_pi.copy_from(&old_pi);
        new_pi.div(&pi_divisor);
        let pi_over_two = new_pi.clone();

        pi_divisor.mult_int(2, 0);
        new_pi.copy_from(&old_pi);
        new_pi.div(&pi_divisor);
        let pi_over_four = new_pi.clone();

        let mut three_pi_over_two = pi.clone();
        three_pi_over_two.mult_int(3, 0);
        three_pi_over_two.div_int(2, 0);

        let mut over_two_pi = two_pi.clone();
        over_two_pi.invert();

        // fill e using Brother's formula (7)
        let mut e_val = BigFloat::from_int(0);
        let mut old_e = BigFloat::from_int(-1);
        let mut denom = BigFloat::from_int(1);
        let mut i = 0i64;
        loop {
            if i >= c_max_iter as i64 {
                break;
            }
            let n = (8 * i * i + 1) * (8 * i - 4) + 5;
            let mut term = BigFloat::from_int(n);
            term.div(&denom);
            e_val.add(&term);
            if e_val.compare(&old_e) == 0 {
                break;
            }
            old_e.copy_from(&e_val);
            let d = (4 * i + 1) * (4 * i + 2) * (4 * i + 3) * (4 * i + 4);
            denom.mult_int(d, 0);
            i += 1;
        }
        assert_bf!(i < c_max_iter as i64);

        // precompute 1/n!
        let over_fact_len = (4 * i + 32) as usize;
        let mut over_fact = Vec::with_capacity(over_fact_len);
        over_fact.push({
            let mut x = BigFloat::from_int(1);
            x.copy_from(&BigFloat::from_int(1));
            x
        });
        let mut fact = BigFloat::from_int(1);
        for j in 1..over_fact_len {
            fact.mult_int(j as i64, 0);
            let mut inv = fact.clone();
            inv.invert();
            over_fact.push(inv);
        }

        // precompute e_power and e_inv_power
        // count how large they need to be
        let mut power = e_val.clone();
        let mut e_power_len: i64 = 1;
        loop {
            power.mult(&power.clone());
            if power.is_special() {
                break;
            }
            e_power_len += 1;
        }

        // e_power[k] = e^(2^k) for k in E_POWER_NEG+1 .. e_power_len-1
        // stored at index (k - E_POWER_NEG)
        let total = (e_power_len - E_POWER_NEG) as usize;
        let mut e_power: Vec<BigFloat> = (0..total).map(|_| BigFloat::new()).collect();
        let mut e_inv_power: Vec<BigFloat> = (0..total).map(|_| BigFloat::new()).collect();

        // index 0 corresponds to k = E_POWER_NEG (i.e., -7)
        // index (0 - E_POWER_NEG) = 7 corresponds to k=0
        let k0_idx = (0 - E_POWER_NEG) as usize;

        e_power[k0_idx].copy_from(&e_val);
        e_inv_power[k0_idx].copy_from(&e_val);
        e_inv_power[k0_idx].invert();

        // fill positive indices: e_power[k] = e_power[k-1]^2
        for k in 1..e_power_len {
            let idx = (k - E_POWER_NEG) as usize;
            let prev_idx = (k - 1 - E_POWER_NEG) as usize;
            let prev = e_power[prev_idx].clone();
            e_power[idx].copy_from(&prev);
            e_power[idx].mult(&prev);
            let prev_inv = e_inv_power[prev_idx].clone();
            e_inv_power[idx].copy_from(&prev_inv);
            e_inv_power[idx].mult(&prev_inv);
        }

        // fill negative indices: e_power[k] = sqrt(e_power[k+1])
        // k goes from -1 down to E_POWER_NEG+1
        let mut k = -1i64;
        while k > E_POWER_NEG {
            let idx = (k - E_POWER_NEG) as usize;
            let next_idx = (k + 1 - E_POWER_NEG) as usize;
            let next = e_power[next_idx].clone();
            e_power[idx].copy_from(&next);
            e_power[idx].sqrt();
            let next_inv = e_inv_power[next_idx].clone();
            e_inv_power[idx].copy_from(&next_inv);
            e_inv_power[idx].sqrt();
            k -= 1;
        }

        Cache {
            e: e_val,
            zero: BigFloat::new(),
            e_power,
            e_inv_power,
            e_power_len,
            pi,
            two_pi,
            over_two_pi,
            pi_over_two,
            three_pi_over_two,
            pi_over_four,
            over_fact,
        }
    }
}
