#![cfg(target_arch = "x86_64")]
#![cfg(target_feature = "avx512f")]
#![feature(portable_simd)]
#![feature(stdarch_x86_avx512)]

pub mod byte_count;
pub mod nucleotide;
pub mod utils;
