// This is insanely hard to make generic atm, so just bear with me
pub const fn parse_u32(s: &str) -> u32 {
    let mut out: u32 = 0;
    let mut i: usize = 0;
    while i < s.len() {
        out *= 10;
        out += (s.as_bytes()[i] - b'0') as u32;
        i += 1;
    }
    out
}

pub const fn parse_u64(s: &str) -> u64 {
    let mut out: u64 = 0;
    let mut i: usize = 0;
    while i < s.len() {
        out *= 10;
        out += (s.as_bytes()[i] - b'0') as u64;
        i += 1;
    }
    out
}

pub const fn parse_usize(s: &str) -> usize {
    let mut out: usize = 0;
    let mut i: usize = 0;
    while i < s.len() {
        out *= 10;
        out += (s.as_bytes()[i] - b'0') as usize;
        i += 1;
    }
    out
}