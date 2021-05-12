//! Crate for producing images of the progressive scan passes found in JPEG
//! image files.
use std::io::{Read, Seek};

/// Jpeg header magic bytes.
pub const JPEG_MAGIC_BYTES: [u8; 3] = [0xFF, 0xD8, 0xFF];
/// Byte padding.
pub const PADDING: u8 = 0x00;
/// Temporary marker.
pub const TEM: u8 = 0x01;
/// End of image marker.
pub const EOI: u8 = 0xD9;
/// Start of scan marker.
pub const SOS: u8 = 0xDA;
/// Fill bytes (markers may be preceded by any number of these).
pub const FILL: u8 = 0xFF;

/// Consume all bytes in the current marker section.
pub fn consume_marker_section<R: Read>(r: &mut R) -> Result<(), std::io::Error> {
    let mut buf = [0, 0];
    r.read_exact(&mut buf)?;

    // Length of marker section includes length bytes, so we subtract 2 bytes
    let marker_length = u16::from_be_bytes(buf).saturating_sub(2);

    // Consume the section's bytes
    for _ in 0..marker_length {
        r.read_exact(&mut buf[..1])?;
    }

    Ok(())
}

/// Consume the bytes in an [SOS] marker section.
pub fn consume_sos_section<R: Read + Seek>(r: &mut R) -> Result<(), std::io::Error> {
    loop {
        match find_next_marker(r)? {
            // Ignore Restart markers, temporary markers, and padding bytes
            PADDING | TEM | 0xD0..=0xD7 => {}
            // Rewind the cursor to process the next marker
            _ => {
                r.seek(std::io::SeekFrom::Current(-2))?;
                return Ok(());
            }
        }
    }
}

/// Consume bytes until the next marker is found.
pub fn find_next_marker<R: Read>(r: &mut R) -> Result<u8, std::io::Error> {
    let mut buf = [0];

    // Find next fill byte
    while buf != [FILL] {
        r.read_exact(&mut buf)?;
    }

    // Consume all of the fill bytes that precede the marker byte
    while buf == [FILL] {
        r.read_exact(&mut buf)?;
    }

    Ok(buf[0])
}
