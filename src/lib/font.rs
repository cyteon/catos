const FONT: &[u8] = include_bytes!("../../assets/font.psfu");

fn u32(offset: usize) -> u32 {
    u32::from_le_bytes(FONT[offset..offset + 4].try_into().unwrap())
}

struct Font {
    pub glyph_start: usize,
    pub bpg: usize,
    pub width: usize,
    pub height: usize,
}

fn parse_font() -> Font {
    assert_eq!(u32(0), 0x864a_b572); // psfu v2

    Font {
        glyph_start: u32(8) as usize,
        bpg: u32(20) as usize,
        width: u32(24) as usize,
        height: u32(28) as usize,
    }
}
