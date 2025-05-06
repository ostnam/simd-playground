use std::arch::asm;
use std::arch::x86_64::_mm512_cmpeq_epu8_mask;
use std::simd::cmp::SimdPartialEq as _;
use std::simd::Simd;

use crate::utils::popcnt_u64;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct NucleotideCount {
    pub a: u64,
    pub c: u64,
    pub g: u64,
    pub t: u64,
}

pub fn count_ref(src: &[u8]) -> NucleotideCount {
    let mut res = NucleotideCount::default();
    for byte in src {
        match byte {
            b'A' => res.a += 1,
            b'C' => res.c += 1,
            b'G' => res.g += 1,
            b'T' => res.t += 1,
            _ => (),
        };
    }
    res
}

pub fn count_asm_naive(src: &[u8]) -> NucleotideCount {
    let Some(last) = src.last() else {
        return NucleotideCount::default();
    };
    let a;
    let c;
    let g;
    let t;
    unsafe {
        asm!(
            "
            xor {a}, {a}
            xor {c}, {c}
            xor {g}, {g}
            xor {t}, {t}
            mov {idx}, {ptr}

            2:
            movzx {cur}, byte ptr [{idx}]
            cmp {cur}, {a_byte}
            jne 3f
            add {a}, 1

            3:
            cmp {cur}, {c_byte}
            jne 4f
            add {c}, 1

            4:
            cmp {cur}, {g_byte}
            jne 5f
            add {g}, 1

            5:
            cmp {cur}, {t_byte}
            jne 6f
            add {t}, 1

            6:
            add {idx}, 1
            cmp {idx}, {last}
            jbe 2b
            ",

            last = in(reg) last,
            ptr = in(reg) src.as_ptr(),
            a_byte = in(reg) b'A' as u64,
            c_byte = in(reg) b'C' as u64,
            g_byte = in(reg) b'G' as u64,
            t_byte = in(reg) b'T' as u64,

            idx = out(reg) _,
            cur = out(reg) _,
            a = out(reg) a,
            c = out(reg) c,
            g = out(reg) g,
            t = out(reg) t,
            options(readonly, nostack),
        );
    }
    NucleotideCount { a, c, g, t, }
}

pub fn count_std_simd(src: &[u8]) -> NucleotideCount {
    let mut res = NucleotideCount::default();
    let all_a = Simd::from_array([b'A'; 64]);
    let all_c = Simd::from_array([b'C'; 64]);
    let all_g = Simd::from_array([b'G'; 64]);
    let all_t = Simd::from_array([b'T'; 64]);
    let all_0 = Simd::from_array([b'\0'; 64]);
    for idx in (0..src.len()).step_by(64) {
        let current = Simd::load_or(&src[idx..], all_0);
        res.a += current.simd_eq(all_a).to_bitmask().count_ones() as u64;
        res.c += current.simd_eq(all_c).to_bitmask().count_ones() as u64;
        res.g += current.simd_eq(all_g).to_bitmask().count_ones() as u64;
        res.t += current.simd_eq(all_t).to_bitmask().count_ones() as u64;
    }
    res
}

pub fn count_simd_intrinsics(src: &[u8]) -> NucleotideCount {
    let mut res = NucleotideCount::default();
    let all_a = Simd::from_array([b'A'; 64]).into();
    let all_c = Simd::from_array([b'C'; 64]).into();
    let all_g = Simd::from_array([b'G'; 64]).into();
    let all_t = Simd::from_array([b'T'; 64]).into();
    let all_0 = Simd::from_array([b'\0'; 64]);
    for idx in (0..src.len()).step_by(64) {
        let current = Simd::load_or(&src[idx..], all_0).into();
        let cmp_a = unsafe {
            _mm512_cmpeq_epu8_mask(current, all_a)
        };
        let cmp_c = unsafe {
            _mm512_cmpeq_epu8_mask(current, all_c)
        };
        let cmp_g = unsafe {
            _mm512_cmpeq_epu8_mask(current, all_g)
        };
        let cmp_t = unsafe {
            _mm512_cmpeq_epu8_mask(current, all_t)
        };
        res.a += popcnt_u64(cmp_a);
        res.c += popcnt_u64(cmp_c);
        res.g += popcnt_u64(cmp_g);
        res.t += popcnt_u64(cmp_t);
    }
    res
}

pub fn count_simd_asm(src: &[u8]) -> NucleotideCount {
    let a;
    let c;
    let g;
    let t;
    unsafe {
        asm!(
            "
            xor {a}, {a}
            xor {c}, {c}
            xor {g}, {g}
            xor {t}, {t}
            xor {eql}, {eql}
            mov {idx}, {ptr}
            mov {last}, {ptr}
            add {last}, {src_len}
            sub {last}, 1

            cmp {src_len}, 64
            jb 3f
            2:
            cmp {idx}, {last_block}
            jge 3f

            vmovdqu8 {cur_zmm}, [{idx}]

            vpcmpeqb {k}, {cur_zmm}, {a_bytes}
            kmovq {eql}, {k}
            popcnt {eql}, {eql}
            add {a}, {eql}

            vpcmpeqb {k}, {cur_zmm}, {c_bytes}
            kmovq {eql}, {k}
            popcnt {eql}, {eql}
            add {c}, {eql}

            vpcmpeqb {k}, {cur_zmm}, {g_bytes}
            kmovq {eql}, {k}
            popcnt {eql}, {eql}
            add {g}, {eql}

            vpcmpeqb {k}, {cur_zmm}, {t_bytes}
            kmovq {eql}, {k}
            popcnt {eql}, {eql}
            add {t}, {eql}

            add {idx}, 64
            jmp 2b
            ",

            "
            3:
            cmp {idx}, {last}
            jg 8f
            movzx {cur}, byte ptr [{idx}]
            cmp {cur}, {a_byte}
            jne 4f
            add {a}, 1

            4:
            cmp {cur}, {c_byte}
            jne 5f
            add {c}, 1

            5:
            cmp {cur}, {g_byte}
            jne 6f
            add {g}, 1

            6:
            cmp {cur}, {t_byte}
            jne 7f
            add {t}, 1

            7:
            add {idx}, 1
            jmp 3b

            8:
            ",
            src_len = in(reg) src.len(),
            last_block = in(reg) (src.len() / 64) * 64,
            ptr = in(reg) src.as_ptr(),
            a_bytes = in(zmm_reg) Simd::from_array([b'A'; 64]),
            c_bytes = in(zmm_reg) Simd::from_array([b'C'; 64]),
            g_bytes = in(zmm_reg) Simd::from_array([b'G'; 64]),
            t_bytes = in(zmm_reg) Simd::from_array([b'T'; 64]),
            a_byte = const b'A' as u64,
            c_byte = const b'C' as u64,
            g_byte = const b'G' as u64,
            t_byte = const b'T' as u64,

            last = out(reg) _,
            k = out(kreg) _,
            cur_zmm = out(zmm_reg) _,
            eql = out(reg) _,
            idx = out(reg) _,
            cur = out(reg) _,
            a = out(reg) a,
            c = out(reg) c,
            g = out(reg) g,
            t = out(reg) t,
            options(readonly, nostack),
        );
    }
    NucleotideCount { a, c, g, t }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::quickcheck;

    quickcheck! {
        fn all_impls_equal(bytes: Vec<u8>) -> bool {
            let ref_ = count_ref(&bytes);
            let asm_naive = count_asm_naive(&bytes);
            let std_simd = count_std_simd(&bytes);
            let simd_intrinsics = count_simd_intrinsics(&bytes);
            let simd_asm = count_simd_asm(&bytes);
            ref_ == asm_naive && ref_ == std_simd && ref_ == simd_intrinsics && ref_ == simd_asm
        }
    }

    #[test]
    fn count_simd_asm_empty() {
        assert_eq!(count_std_simd(b""), NucleotideCount::default());
    }

    #[test]
    fn count_simd_asm_one() {
        assert_eq!(count_std_simd(b"A"), NucleotideCount { a: 1, ..NucleotideCount::default() });
    }

    #[test]
    fn count_simd_asm_zero() {
        assert_eq!(count_std_simd(b"\0"), NucleotideCount { a: 0, ..NucleotideCount::default() });
    }

    #[test]
    fn count_simd_asm_full_block() {
        assert_eq!(count_std_simd(&[b'A'; 64]), NucleotideCount { a: 64, ..NucleotideCount::default() });
    }

    #[test]
    fn count_simd_asm_full_block_plus_one() {
        assert_eq!(count_std_simd(&[b'A'; 65]), NucleotideCount { a: 65, ..NucleotideCount::default() });
    }

    #[test]
    fn count_nucleotides_std_simd_empty() {
        assert_eq!(count_std_simd(b""), NucleotideCount::default());
    }

    #[test]
    fn count_nucleotides_std_simd_one() {
        assert_eq!(
            count_std_simd(&[b'A'; 64]),
            NucleotideCount { a: 64, ..NucleotideCount::default() },
        );
        assert_eq!(
            count_std_simd(&[b'C'; 64]),
            NucleotideCount { c: 64, ..NucleotideCount::default() },
        );
        assert_eq!(
            count_std_simd(&[b'G'; 64]),
            NucleotideCount { g: 64, ..NucleotideCount::default() },
        );
        assert_eq!(
            count_std_simd(&[b'T'; 64]),
            NucleotideCount { t: 64, ..NucleotideCount::default() },
        );
    }

    #[test]
    fn count_nucleotides_std_simd_one_each() {
        assert_eq!(
            count_std_simd(b"ACGT"),
            NucleotideCount { a: 1, c: 1, g: 1, t: 1},
        );
    }
}
