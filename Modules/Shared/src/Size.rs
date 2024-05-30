#![allow(non_camel_case_types)]
use num::{Num, NumCast, ToPrimitive, Unsigned};

pub trait Size_trait: Unsigned + Num + NumCast + PartialOrd + ToPrimitive + Copy {}

impl Size_trait for u32 {}

impl Size_trait for u64 {}

impl Size_trait for u128 {}

impl Size_trait for usize {}
