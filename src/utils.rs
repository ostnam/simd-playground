use std::arch::asm;

#[inline(always)]
pub fn popcnt_u64(x: u64) -> u64 {
    let res;
    unsafe {
        asm!(
            "popcnt {out}, {x}",
            x = in(reg) x,
            out = out(reg) res,
        );
    };
    res
}
