const fn f32_abs(a: f32) -> f32 {
    if a < 0.0 {
        -a
    } else {
        a
    }
}

const SIGN_MASK: u32 = 0b1000_0000_0000_0000_0000_0000_0000_0000;
const EXPONENT_MASK: u32 = 0b0111_1111_1000_0000_0000_0000_0000_0000;
const MANTISSA_MASK: u32 = 0b0000_0000_0111_1111_1111_1111_1111_1111;

const fn f32_exponent_value(a: f32) -> i32 {
    let bits = (a.to_bits() & EXPONENT_MASK).overflowing_shr(23).0;

    (bits as i32) - 127
}

const fn f32_ln(a: f32) -> f32 {
    if f32_abs(a - 1.0) < f32::EPSILON {
        return 0.0;
    }

    let x_less_than_1 = a < 1.0;

    let x_working = if x_less_than_1 { 1.0 / a } else { a };

    let base2_exponent = f32_exponent_value(x_working) as u32;
    let divisor = f32::from_bits(x_working.to_bits() & EXPONENT_MASK);

    let x_working = x_working / divisor;

    let ln_1to2_polynomial = -1.741_793_9
        + (2.821_202_6
            + (-1.469_956_8 + (0.447_179_55 - 0.056_570_851 * x_working) * x_working) * x_working)
            * x_working;

    let result = (base2_exponent as f32) * core::f32::consts::LN_2 + ln_1to2_polynomial;

    if x_less_than_1 {
        -result
    } else {
        result
    }
}

const fn f32_copysign(a: f32, sign: f32) -> f32 {
    let source_bits = sign.to_bits();
    let source_sign = source_bits & SIGN_MASK;
    let signless_dest_bits = a.to_bits() & !SIGN_MASK;
    f32::from_bits(signless_dest_bits | source_sign)
}

const fn f32_fract(a: f32) -> f32 {
    let x_bits = a.to_bits();
    let exponent = f32_exponent_value(a);

    if exponent < 0 {
        return a;
    }

    let fractional_part = x_bits.overflowing_shl(exponent as u32).0 & MANTISSA_MASK;

    if fractional_part == 0 {
        return 0.0;
    }

    let exponent_shift = (fractional_part.leading_zeros() - (32 - 23)) + 1;

    let fractional_normalized = fractional_part.overflowing_shl(exponent_shift).0 & MANTISSA_MASK;

    let new_exponent_bits = (127 - exponent_shift).overflowing_shl(23).0;

    f32_copysign(f32::from_bits(fractional_normalized | new_exponent_bits), a)
}

const fn f32_trunc(a: f32) -> f32 {
    let x_bits = a.to_bits();
    let exponent = f32_exponent_value(a);

    if exponent < 0 {
        return 0.0;
    }

    let exponent_clamped = if exponent < 0 { 0 } else { exponent as u32 };

    let fractional_part = x_bits.overflowing_shl(exponent_clamped).0 & MANTISSA_MASK;

    if fractional_part == 0 {
        return a;
    }

    let fractional_mask = fractional_part.overflowing_shr(exponent_clamped).0;

    f32::from_bits(x_bits & !fractional_mask)
}

const fn f32_exp_smallx(a: f32) -> f32 {
    let total = 1.0;
    let total = 1.0 + (a / 4.0) * total;
    let total = 1.0 + (a / 3.0) * total;
    let total = 1.0 + (a / 2.0) * total;
    let total = 1.0 + (a / 1.0) * total;
    total
}

const fn f32_set_exponent(a: f32, exponent: i32) -> f32 {
    let without_exponent = a.to_bits() & !EXPONENT_MASK;
    let only_exponent = ((exponent + 127) as u32).overflowing_shl(23).0;

    f32::from_bits(without_exponent | only_exponent)
}

const fn f32_exp(a: f32) -> f32 {
    if a == 0.0 {
        return 1.0;
    }

    if f32_abs(a - 1.0) < f32::EPSILON {
        return core::f32::consts::E;
    }

    if f32_abs(a - -1.0) < f32::EPSILON {
        return 1.0 / core::f32::consts::E;
    }

    let x_ln2recip = a * core::f32::consts::LOG2_E;
    let x_fract = f32_fract(x_ln2recip);
    let x_trunc = f32_trunc(x_ln2recip);

    let x_fract = x_fract * core::f32::consts::LN_2;
    let fract_exp = f32_exp_smallx(x_fract);

    let fract_exponent = f32_exponent_value(fract_exp).saturating_add(x_trunc as i32);

    if fract_exponent < -127 {
        return 0.0;
    }

    if fract_exponent > 127 {
        return f32::INFINITY;
    }

    f32_set_exponent(fract_exp, fract_exponent)
}

const fn f32_powf(a: f32, n: f32) -> f32 {
    if a > 0.0 {
        let y = f32_ln(a);
        f32_exp(n * f32_ln(a))
    } else if a == 0.0 {
        return 0.0;
    } else {
        panic!("no")
    }
}

pub const fn gamma_curve<const SIZE: usize>(gamma: f32) -> [u8; SIZE] {
    let mut result = [0; SIZE];

    let mut i = 0;
    while i < SIZE {
        let gamma_corrected = f32_powf(i as f32 / SIZE as f32, gamma) * SIZE as f32 + 0.3;

        result[i] = gamma_corrected as u8;

        i += 1;
    }

    result
}
