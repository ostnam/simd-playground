use std::arch::asm;

pub fn count_bytes_ref(
    bytes: &[u8],
    tgt: u8,
) -> usize {
    bytes.into_iter().filter(|x| **x == tgt).count()
}

pub fn count_bytes_x86(
    bytes: &[u8],
    tgt: u8,
) -> usize {
    let Some(last) = bytes.last() else {
        return 0;
    };
    let res;
    unsafe {
        asm!(
            "
                mov {idx}, {ptr}
                xor {res}, {res}

                2:
                xor {eql}, {eql}
                movzx {cur},  byte ptr [{idx}]
                cmp {cur}, {tgt}
                cmove {eql}, {one}
                add {res}, {eql}
                add {idx}, 1
                cmp {idx}, {end}
                jbe 2b
            ",
            one = in(reg) 1_u64,
            ptr = in(reg) bytes.as_ptr(),
            end = in(reg) last as *const u8,
            tgt = in(reg) tgt as u64,

            idx = out(reg) _,
            eql = out(reg) _,
            cur = out(reg) _,
            res = out(reg) res,
            options(readonly, nostack),
        );
    };
    res
}

pub fn count_bytes_x86_jne(
    bytes: &[u8],
    tgt: u8,
) -> usize {
    let Some(last) = bytes.last() else {
        return 0;
    };
    let res;
    unsafe {
        asm!(
            "
                mov {idx}, {ptr}
                xor {res}, {res}

                2:
                movzx {cur},  byte ptr [{idx}]
                cmp {cur}, {tgt}
                jne 3f
                add {res}, 1

                3:
                add {idx}, 1
                cmp {idx}, {end}
                jbe 2b
            ",
            ptr = in(reg) bytes.as_ptr(),
            end = in(reg) last as *const u8,
            tgt = in(reg) tgt as u64,

            idx = out(reg) _,
            cur = out(reg) _,
            res = out(reg) res,
            options(readonly, nostack),
        );
    };
    res
}


#[cfg(test)]
mod tests {
    use quickcheck::quickcheck;

    use super::*;

    #[test]
    fn count_bytes_x86_empty() {
        assert_eq!(
            count_bytes_x86(&[], 0),
            0,
        );
    }

    #[test]
    fn count_bytes_one() {
        assert_eq!(
            count_bytes_x86(&[1], 0),
            0,
        );
    }

    #[test]
    fn count_bytes_one_eq() {
        assert_eq!(
            count_bytes_x86(&[2], 0),
            0,
        );
    }

    #[test]
    fn long_case_no_error() {
        let mut buf = [0; 2048];
        rand::fill(&mut buf);
        count_bytes_x86(&buf, 128);
    }

    quickcheck! {
        fn x86_equals_reference_impl(bytes: Vec<u8>, tgt: u8) -> bool {
            count_bytes_ref(&bytes, tgt) == count_bytes_x86(&bytes, tgt)
        }
    }
}
