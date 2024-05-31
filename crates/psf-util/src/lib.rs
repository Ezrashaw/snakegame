use core::slice;
use std::{
    io::{self, Write},
    process::{Command, Stdio},
    str::Lines,
};

pub const MAGIC_BYTES: u32 = 0x864ab572;

pub fn read_psf(mut psf: &[u8]) -> PsfFont {
    assert!(psf.len() > 32);

    let mut next_u32 = || {
        let b4 = &psf[0..4];
        psf = &psf[4..];
        u32::from_le_bytes(b4.try_into().unwrap())
    };

    assert!(next_u32() == MAGIC_BYTES); // magic bytes
    assert!(next_u32() == 0); // version
    assert!(next_u32() == 32); // header length in bytes
    assert!(next_u32() == 1); // flags (has unicode table)

    let glyph_count = next_u32();
    let bytes_per_glyph = next_u32();
    let height = next_u32();
    let width = next_u32();
    let stride = (width + 7) / 8;

    let total_glyph_bytes = (stride * height * glyph_count) as usize;
    let glyphs = &psf[0..total_glyph_bytes];
    psf = &psf[total_glyph_bytes..];

    let mut unicode_table = Vec::with_capacity(glyph_count as usize);
    for _ in 0..glyph_count {
        unicode_table.push(UnicodeEntry::read_from_bytes(&mut psf));
    }

    PsfFont {
        glyph_count,
        bytes_per_glyph,
        height,
        width,
        glyphs,
        unicode_table,
    }
}

pub struct PsfFont<'a> {
    glyph_count: u32,
    bytes_per_glyph: u32,
    height: u32,
    width: u32,
    glyphs: &'a [u8],
    unicode_table: Vec<UnicodeEntry>,
}

impl<'a> PsfFont<'a> {
    pub fn write_psf(&self, w: &mut impl io::Write) -> io::Result<()> {
        let mut write_u32 = |val: u32| w.write_all(&val.to_ne_bytes());

        write_u32(MAGIC_BYTES)?; // magic bytes
        write_u32(0)?; // version
        write_u32(32)?; // header length in bytes
        write_u32(1)?; // flags (has unicode table)

        write_u32(self.glyph_count)?;
        write_u32(self.bytes_per_glyph)?;
        write_u32(self.height)?;
        write_u32(self.width)?;

        w.write_all(self.glyphs)?;

        for entry in &self.unicode_table {
            w.write_all(String::from_iter(entry.get_singles()).as_bytes())?;
            w.write_all(&[0xFF])?;
        }

        Ok(())
    }

    pub fn glyph_count(&self) -> u32 {
        self.glyph_count
    }

    pub fn stride(&self) -> usize {
        ((self.width + 7) / 8) as usize
    }

    pub fn print_table(&self) {
        let per_row = oca_io::get_termsize().0 as u32 / (self.width + 2);
        for row in 0..(self.glyph_count / per_row) {
            for g in 0..per_row {
                let g = g + row * per_row;
                print!("  {g:#04x}    ");
            }
            println!();

            for c_row in 0..(self.height as usize / 2) {
                for g in 0..per_row {
                    self.print_double_row(row * per_row + g, c_row * 2);
                    print!("  ");
                }
                println!();
            }

            for g in 0..per_row {
                let g = g + row * per_row;
                let table = &self.unicode_table[g as usize];
                print!("{:^8}  ", table.singles[0].escape_debug().to_string());
            }

            println!("\n");
        }
    }

    pub fn print_glyph(&self, g: u32) {
        println!("  {g:#04x}");
        for c_row in 0..(self.height as usize / 2) {
            self.print_double_row(g, c_row * 2);
            println!();
        }
        let table = &self.unicode_table[g as usize];
        for uni in table.get_singles() {
            print!(" {}", uni.escape_debug());
        }
        println!();
    }

    fn print_double_row(&self, g: u32, first_r: usize) {
        let stride = self.stride();
        let gbytes = self.get_glyph(g);
        let start = first_r * stride;
        let top = &gbytes[start..(start + stride)];
        let btm = &gbytes[(start + stride)..];
        print!("\x1B[100m");
        for byte in 0..stride {
            for bit in (0..8).rev() {
                let mask = 0x1 << bit;
                let char = match (top[byte] & mask != 0, btm[byte] & mask != 0) {
                    (true, true) => '█',
                    (true, false) => '▀',
                    (false, true) => '▄',
                    (false, false) => ' ',
                };
                print!("{char}");
            }
        }
        print!("\x1B[0m");
    }

    fn get_glyph(&self, i: u32) -> &[u8] {
        assert!(i <= self.glyph_count);

        let offset = (self.bytes_per_glyph * i) as usize;
        &self.glyphs[offset..(offset + self.bytes_per_glyph as usize)]
    }
}

pub struct UnicodeEntry {
    singles: [char; 16],
    singles_count: usize,
}

impl UnicodeEntry {
    pub fn get_singles(&self) -> &[char] {
        &self.singles[0..self.singles_count]
    }

    fn read_from_bytes(bytes: &mut &[u8]) -> Self {
        let mut singles = ['\0'; 16];
        let mut singles_count = 0;
        while bytes[0] != 0xFE && bytes[0] != 0xFF {
            let boundary = find_utf8_boundary(bytes);
            singles[singles_count] = std::str::from_utf8(&bytes[0..boundary])
                .unwrap()
                .chars()
                .next()
                .unwrap();
            singles_count += 1;
            *bytes = &bytes[boundary..];
        }

        if bytes[0] == 0xFE {
            println!("WARNING: unsupported unicode character sequence");
            while bytes[0] != 0xFF {
                *bytes = &bytes[1..];
            }
        }

        *bytes = &bytes[1..];

        Self {
            singles,
            singles_count,
        }
    }
}

fn find_utf8_boundary(bytes: &[u8]) -> usize {
    if bytes[0] & 0b1000_0000 == 0 {
        1
    } else {
        let mut i = 1;
        loop {
            if bytes[i] >> 6 != 0b10 {
                break i;
            }

            i += 1;
        }
    }
}

pub fn ungzip(bytes: &[u8]) -> Vec<u8> {
    let mut cmd = Command::new("gzip")
        .arg("-d")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");

    cmd.stdin.as_mut().unwrap().write_all(bytes).unwrap();
    cmd.wait_with_output().unwrap().stdout
}

pub fn psf2txt(w: &mut impl io::Write, filename: &str, psf: PsfFont) -> io::Result<()> {
    writeln!(w, "load {filename}")?;
    writeln!(w, "size {}x{}\n", psf.width, psf.height)?;

    for i in 0..psf.glyph_count {
        writeln!(w, "character {i:#04x}")?;
        write!(w, "unicode")?;
        for ch in psf.unicode_table[i as usize].get_singles() {
            write!(w, " '{}'", ch.escape_debug())?;
        }
        writeln!(w)?;

        let glyph = psf.get_glyph(i);
        for row in glyph.chunks(psf.stride()) {
            for byte in row {
                for bit in (0..8).rev() {
                    let mask = 0x1 << bit;
                    write!(w, "{}", if byte & mask == 0 { "-" } else { "#" })?;
                }
            }
            writeln!(w)?;
        }
        writeln!(w)?;
    }

    Ok(())
}

pub fn txt2psf(mut lines: Lines, psf: &mut PsfFont) -> io::Result<()> {
    let Some(("size", size)) = lines.next().unwrap().split_once(' ') else {
        panic!("expected \"size <width>x<height>\"");
    };

    let Some((width, height)) = size.split_once('x') else {
        panic!("expected \"size <width>x<height>\"");
    };

    let (width, height): (u32, u32) = (width.parse().unwrap(), height.parse().unwrap());
    assert!(width == psf.width);
    assert!(height == psf.height);

    while let Some(line) = lines.next() {
        if line.trim().is_empty() {
            continue;
        }

        let Some(("character 0", char)) = line.split_once('x') else {
            panic!("expected \"character 0x<character>\"");
        };

        let char = u32::from_str_radix(char, 16).unwrap();

        let Some(("unicode ", unicode)) = lines.next().unwrap().split_once('\'') else {
            panic!("expected \"unicode '<character>'...");
        };

        let table = &mut psf.unicode_table[char as usize];
        table.singles_count = 0;
        for char in unicode.split("' '") {
            let ch = char.chars().next().unwrap();
            table.singles[table.singles_count] = ch;
            table.singles_count += 1;
        }

        let data = psf.get_glyph(char);
        // SAFETY: no no no no no (but it works ;)
        let data = unsafe { slice::from_raw_parts_mut(data.as_ptr() as *mut u8, data.len()) };
        assert!(data.len() == height as usize); // TODO: one of the many things to fix

        for h in 0..height {
            data[h as usize] = 0x0;
            let line = lines.next().unwrap();
            assert!(line.trim().len() == width as usize);

            let mut mask = 0b1000_0000;
            for ch in line.chars() {
                match ch {
                    '#' => data[h as usize] |= mask,
                    '-' => (),
                    _ => panic!("unexpected character"),
                }
                mask >>= 1;
            }
        }

        psf.print_glyph(char);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    #[test]
    fn test_default_font() {
        let bytes =
            super::ungzip(&fs::read("/usr/share/kbd/consolefonts/default8x16.psfu.gz").unwrap());

        let font = super::read_psf(&bytes);

        let mut buf = Vec::with_capacity(bytes.len());
        font.write_psf(&mut buf).unwrap();

        assert_eq!(bytes, buf);
    }
}
