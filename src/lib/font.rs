pub const FONT: &[u8] = include_bytes!("../../assets/font.psfu");

fn u32(offset: usize) -> u32 {
    u32::from_le_bytes(FONT[offset..offset + 4].try_into().unwrap())
}

pub struct Font {
    pub glyph_start: usize,
    pub bpg: usize,
    pub height: usize,
    pub width: usize,
}

pub fn parse_font() -> Font {
    if u32(0) == 0x864a_b572 {
        // psf2
        Font {
            glyph_start: u32(8) as usize,
            bpg: u32(20) as usize,
            height: u32(24) as usize,
            width: u32(28) as usize,
        }
    } else {
        // psf1
        let charsize = FONT[3] as usize;

        Font {
            glyph_start: 4,
            bpg: charsize,
            height: charsize,
            width: 8,
        }
    }
}
