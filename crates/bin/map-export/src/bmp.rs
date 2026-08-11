//! Minimal 24-bit BMP writer — zero dependencies, opens natively on Windows.

use std::io::{self, Write};
use std::path::Path;

/// Write `rgb` (row-major, top-down) as a 24bpp BMP.
pub fn write(path: &Path, width: u32, height: u32, rgb: &[(u8, u8, u8)]) -> io::Result<()> {
    assert_eq!(rgb.len(), (width as usize) * (height as usize));
    let row_bytes = (width * 3).next_multiple_of(4);
    let pixel_bytes = row_bytes * height;
    let file_size = 54 + pixel_bytes;

    let mut out = Vec::with_capacity(file_size as usize);
    // BITMAPFILEHEADER
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&file_size.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&54u32.to_le_bytes());
    // BITMAPINFOHEADER
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&width.to_le_bytes());
    out.extend_from_slice(&height.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&24u16.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&pixel_bytes.to_le_bytes());
    out.extend_from_slice(&2835u32.to_le_bytes());
    out.extend_from_slice(&2835u32.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    // Pixel rows, bottom-up, BGR, padded to 4 bytes.
    let padding = (row_bytes - width * 3) as usize;
    for y in (0..height).rev() {
        for x in 0..width {
            let (r, g, b) = rgb[(y as usize) * (width as usize) + (x as usize)];
            out.extend_from_slice(&[b, g, r]);
        }
        out.extend_from_slice(&[0u8; 3][..padding]);
    }

    let mut file = std::fs::File::create(path)?;
    file.write_all(&out)
}
