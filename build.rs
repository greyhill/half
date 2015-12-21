use std::fs;
use std::path;
use std::iter;
use std::io::Write;
use std::mem;
use std::slice;

fn slice_cast<T: Sized>(i: &[T]) -> &[u8] {
    unsafe {
        let p: *const u8 = mem::transmute(&i[0]);
        slice::from_raw_parts(p, i.len() * mem::size_of::<T>())
    }
}

fn setup_exponent_table<P: AsRef<path::Path>>(p: P) -> () {
    let mut table: Vec<u32> = iter::repeat(0u32).take(64).collect();
    table[0] = 0;

    for i in 1..31 {
        table[i] = ((i as u32) << 23) as u32;
    }
    table[31] = 0x47800000;
    table[32] = 0x80000000;
    for i in 33..63 {
        table[i] = (0x80000000 + (i as u32 - 32) << 23) as u32;
    }
    table[63] = 0xC7800000;

    let mut file = fs::File::create(p.as_ref()).expect("Error opening exponent table");
    file.write(slice_cast(&table[..])).expect("Error writing exponent table");
}

fn setup_offset_table<P: AsRef<path::Path>>(p: P) -> () {
    let mut offset_table: Vec<u32> = iter::repeat(0u32).take(64).collect();
    for i in 0..64 {
        offset_table[i] = 1024;
    }
    offset_table[0] = 0;
    offset_table[32] = 0;

    let mut file = fs::File::create(p.as_ref()).expect("Error opening offset table");
    file.write(slice_cast(&offset_table[..])).expect("Error writing offset table");
}

fn convert_mantissa(i: u32) -> u32 {
    let mut m: u32 = i << 13;
    let mut e: i32 = 0;
    while (m & 0x00800000) == 0 {
        e -= 0x00800000;
        m <<= 1;
    }
    m &= 0xFF7FFFFF;
    e += 0x38800000;
    m | unsafe { mem::transmute::<i32, u32>(e) }
}

fn setup_mantissa_table<P: AsRef<path::Path>>(p: P) -> () {
    let mut mantissa_table: Vec<u32> = iter::repeat(0u32).take(2048).collect();
    mantissa_table[0] = 0;
    for i in 1..1024 {
        mantissa_table[i] = convert_mantissa(i as u32);
    }
    for i in 1024..2048 {
        mantissa_table[i] = 0x38000000 + (((i as u32) - 1024) << 13) as u32;
    }

    let mut file = fs::File::create(p.as_ref()).expect("Error opening mantissa table");
    file.write(slice_cast(&mantissa_table[..])).expect("Error writing mantissa table");
}

fn setup_shift_base_table<P1: AsRef<path::Path>, P2: AsRef<path::Path>>(p1: P1, p2: P2) -> () {
    let mut shift_table: Vec<u8> = iter::repeat(0).take(512).collect();
    let mut base_table: Vec<u16> = iter::repeat(0).take(512).collect();

    for i in 0 .. 256u32 {
        let e: i32 = i as i32 - 127;
        let i0 = (i | 0x000) as usize;
        let i1 = (i | 0x100) as usize;
        if e < -24 {
            base_table[i0] = 0x0000;
            base_table[i1] = 0x8000;
            shift_table[i0] = 24;
            shift_table[i1] = 24;
        } else if e < -14 {
            base_table[i0] = 0x0400 >> (-e - 14);
            base_table[i1] = (0x0400 >> (-e - 14)) | 0x8000;
            shift_table[i0] = (-e-1) as u8;
            shift_table[i1] = (-e-1) as u8;
        } else if e <= 15 {
            base_table[i0] = ((e+15)<<10) as u16;
            base_table[i1] = (((e+15)<<10) | 0x8000) as u16;
            shift_table[i0] = 13;
            shift_table[i1] = 13;
        } else if e < 128 {
            base_table[i0] = 0x7C00;
            base_table[i1] = 0xFC00;
            shift_table[i0] = 24;
            shift_table[i1] = 24;
        } else {
            base_table[i0] = 0x7C00;
            base_table[i1] = 0xFC00;
            shift_table[i0] = 13;
            shift_table[i1] = 13;
        }
    }

    let mut file = fs::File::create(p1.as_ref()).expect("Error opening shift table");
    file.write(slice_cast(&shift_table[..])).expect("Error writing shift table");

    file = fs::File::create(p2.as_ref()).expect("Error opening base table");
    file.write(slice_cast(&base_table[..])).expect("Error writing base table");
}

pub fn main() {
    setup_exponent_table("src/exponent_table.bin");
    setup_offset_table("src/offset_table.bin");
    setup_mantissa_table("src/mantissa_table.bin");
    setup_shift_base_table("src/shift_table.bin", "src/base_table.bin");
}
