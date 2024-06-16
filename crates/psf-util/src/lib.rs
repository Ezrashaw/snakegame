#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]
#![feature(custom_test_frameworks)]
#![test_runner(datatest::runner)]

use std::{
    io::{self, Write},
    process::{Command, Stdio},
    str::Lines,
};

pub const MAGIC_BYTES: u32 = 0x864a_b572;

pub struct PsfFont {
    glyph_count: u32,
    bytes_per_glyph: u32,
    height: u32,
    width: u32,
    glyphs: Vec<u8>,
    unicode_table: Vec<UnicodeEntry>,
}

impl TryFrom<&[u8]> for PsfFont {
    type Error = ();

    fn try_from(mut bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut next_u32 = || {
            let (val, rest) = bytes.split_at(4);
            bytes = rest;
            u32::from_le_bytes(val.try_into().unwrap())
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
        let (glyphs, rest) = bytes.split_at(total_glyph_bytes);
        bytes = rest;

        let mut unicode_table = Vec::with_capacity(glyph_count as usize);
        for _ in 0..glyph_count {
            unicode_table.push(UnicodeEntry::read_from_bytes(&mut bytes));
        }

        Ok(Self {
            glyph_count,
            bytes_per_glyph,
            height,
            width,
            glyphs: glyphs.to_vec(),
            unicode_table,
        })
    }
}

impl PsfFont {
    pub fn write_to(&self, w: &mut impl io::Write) -> io::Result<()> {
        let mut write_u32 = |val: u32| w.write_all(&val.to_ne_bytes());

        write_u32(MAGIC_BYTES)?; // magic bytes
        write_u32(0)?; // version
        write_u32(32)?; // header length in bytes
        write_u32(1)?; // flags (has unicode table)

        write_u32(self.glyph_count)?;
        write_u32(self.bytes_per_glyph)?;
        write_u32(self.height)?;
        write_u32(self.width)?;

        w.write_all(&self.glyphs)?;

        for entry in &self.unicode_table {
            w.write_all(String::from_iter(entry.singles()).as_bytes())?;
            w.write_all(&[0xFF])?;
        }

        Ok(())
    }

    pub fn double_size(&mut self) {
        assert!(self.width == 8);
        let mut glyphs = Vec::with_capacity((self.bytes_per_glyph * self.glyph_count * 4) as usize);
        for glyph in self.glyphs.chunks(self.height as usize) {
            for row in glyph {
                let mut doubled = 0u16;
                for i in 0..8 {
                    let bit = (row >> (7 - i)) & 1;
                    doubled |= u16::from(bit) << ((7 - i) * 2 + 1);
                    doubled |= u16::from(bit) << ((7 - i) * 2);
                }

                glyphs.extend_from_slice(&doubled.to_be_bytes());
                glyphs.extend_from_slice(&doubled.to_be_bytes());
            }
        }

        self.width *= 2;
        self.height *= 2;
        self.bytes_per_glyph *= 4;
        self.glyphs = glyphs;
    }

    #[must_use]
    pub const fn glyph_count(&self) -> u32 {
        self.glyph_count
    }

    #[must_use]
    pub const fn stride(&self) -> usize {
        ((self.width + 7) / 8) as usize
    }

    pub fn print_table(&self) {
        let per_row = u32::from(oca_io::get_termsize().unwrap().0) / (self.width + 2);
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
        for uni in table.singles() {
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

    #[must_use] pub fn get_glyph(&self, i: u32) -> &[u8] {
        let offset = (self.bytes_per_glyph * i) as usize;
        &self.glyphs[offset..(offset + self.bytes_per_glyph as usize)]
    }

    pub fn get_glyph_mut(&mut self, i: u32) -> &mut [u8] {
        let offset = (self.bytes_per_glyph * i) as usize;
        &mut self.glyphs[offset..(offset + self.bytes_per_glyph as usize)]
    }
}

pub struct UnicodeEntry {
    singles: Vec<char>,
}

impl UnicodeEntry {
    #[must_use]
    pub fn singles(&self) -> &[char] {
        &self.singles
    }

    fn read_from_bytes(bytes: &mut &[u8]) -> Self {
        let end_pos = bytes.iter().position(|b| matches!(b, 0xFE | 0xFF)).unwrap();
        let (singles, rest) = bytes.split_at(end_pos);
        *bytes = rest;

        let singles = std::str::from_utf8(singles)
            .unwrap()
            .chars()
            .collect::<Vec<char>>();

        assert!(bytes[0] != 0xFE, "WARNING: unsupported unicode character sequence");

        // Skip the final 0xFF.
        *bytes = &bytes[1..];

        Self { singles }
    }
}

#[must_use]
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

pub fn psf2txt(w: &mut impl io::Write, filename: &str, psf: &PsfFont) -> io::Result<()> {
    writeln!(w, "load {filename}")?;
    writeln!(w, "size {}x{}\n", psf.width, psf.height)?;

    for i in 0..psf.glyph_count {
        writeln!(w, "character {i:#04x}")?;
        write!(w, "unicode")?;
        for ch in psf.unicode_table[i as usize].singles() {
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
        table.singles.clear();
        for char in unicode.split("' '") {
            table.singles.push(char.chars().next().unwrap());
        }

        let data = psf.get_glyph_mut(char);
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
    #[datatest::files("/usr/share/kbd/consolefonts/", { input in r"^(.*).psfu.gz" })]
    fn test_font(input: &[u8]) {
        let bytes = super::ungzip(input);
        // ignore the test if it is version 1 PSF
        if u16::from_le_bytes(bytes[0..2].try_into().unwrap()) == 0x0436 {
            return;
        }

        let font = super::PsfFont::try_from(bytes.as_slice()).unwrap();

        let mut buf = Vec::with_capacity(bytes.len());
        font.write_to(&mut buf).unwrap();

        assert_eq!(bytes, buf);
    }
}
