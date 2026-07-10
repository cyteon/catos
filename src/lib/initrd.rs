use spin::Once;

pub static INITRD: Once<&'static [u8]> = Once::new();

pub fn init(data: &'static [u8]) {
    INITRD.call_once(|| data);
}

pub fn get() -> Option<&'static [u8]> {
    INITRD.get().copied()
}

pub struct File<'a> {
    pub name: &'a str,
    pub data: &'a [u8],
}

fn parse_octal(field: &[u8]) -> usize {
    let mut value = 0;

    for &byte in field {
        match byte {
            b'0'..=b'7' => value = value * 8 + (byte - b'0') as usize,
            _ => break,
        }
    }

    value
}

pub fn files(tar: &[u8]) -> impl Iterator<Item = File<'_>> {
    let mut offset = 0;

    core::iter::from_fn(move || {
        loop {
            if offset + 512 > tar.len() {
                return None;
            }

            let header = &tar[offset..offset + 512];
            if header[0] == 0 {
                return None;
            }

            let size = parse_octal(&header[124..136]);

            let name_raw = &header[0..100];
            let name_end = name_raw.iter().position(|&b| b == 0).unwrap_or(100);
            let name = core::str::from_utf8(&name_raw[..name_end]).unwrap_or("?");
            let name = name.strip_prefix("./").unwrap_or(name);

            let typeflag = header[156];

            let data_start = offset + 512;
            offset = data_start + size.div_ceil(512) * 512;

            if typeflag == b'0' || typeflag == 0 {
                return Some(File {
                    name,
                    data: &tar[data_start..data_start + size],
                });
            }
        }
    })
}

pub fn find<'a>(tar: &'a [u8], name: &str) -> Option<File<'a>> {
    files(tar).find(|file| file.name == name)
}
