//! # Half-precision conversion for Rust
//! This crate uses the fast table-based f32 <-> f16 conversions given in
//! "Fast Half Float Conversions" by Jeroen van der Zijp; see
//! `ftp://ftp.fox-toolkit.org/pub/fasthalffloatconversion.pdf`.

use std::mem::transmute;

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
pub struct f16 {
    pub bytes: u16,
}

impl PartialEq for f16 {
    fn eq(self: &f16, other: &f16) -> bool {
        return self.bytes == other.bytes;
    }
}

impl From<f32> for f16 {
    fn from(f: f32) -> f16 {
        unsafe {
            let base_table: *const u16 = transmute(include_bytes!("base_table.bin"));
            let shift_table: *const u8 = transmute(include_bytes!("shift_table.bin"));

            let ff: u32 = transmute(f);

            let bytes =
                *base_table.offset(((ff >> 23) & 0x1FF) as isize) +
                ((ff & 0x007FFFFF) >> *shift_table.offset(((ff >> 23) & 0x1FF) as isize)) as u16;
            f16 { bytes: bytes }
        }
    }
}

impl Into<f32> for f16 {
    fn into(self: f16) -> f32 {
        unsafe {
            let mantissa_table: *const u32 = transmute(include_bytes!("mantissa_table.bin"));
            let offset_table: *const u32 = transmute(include_bytes!("offset_table.bin"));
            let exponent_table: *const u32 = transmute(include_bytes!("exponent_table.bin"));

            let h0 = (self.bytes >> 10) as isize;
            let h1 = (self.bytes & 0x3FF) as u16 as isize;

            let mt_offset = *offset_table.offset(h0) as isize + h1;
            if h0 & 0x8000 == 0 {
                transmute(*mantissa_table.offset(mt_offset) + *exponent_table.offset(h0))
            } else {
                -1f32 * transmute::<u32, f32>(*mantissa_table.offset(mt_offset) + *exponent_table.offset(h0))
            }
        }
    }
}

pub fn slice_to_f16(v: &[f32]) -> Vec<f16> {
    let mut tr = Vec::with_capacity(v.len());
    for it in v.iter() {
        tr.push(f16::from(*it));
    }
    tr
}

pub fn slice_to_f32(v: &[f16]) -> Vec<f32> {
    let mut tr = Vec::with_capacity(v.len());
    for it in v.iter() {
        tr.push((*it).into());
    }
    tr
}

#[test]
fn test() {
    for bytes in 0..(1 << 15) as u16 {
        let h0 = f16 { bytes: bytes };

        let f: f32 = h0.into();
        let h1 = f16::from(f);
        let f1: f32 = h1.into();
        assert_eq!(f, f1);
    }
}
