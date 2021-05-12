use jpegscans::{
    consume_marker_section, consume_sos_section, find_next_marker, EOI, FILL, JPEG_MAGIC_BYTES, SOS,
};

const HELP_MESSAGE: &str = "jpegscans
Produce scan images from a progressive JPEG file.

USAGE:
    jpegscans [input] [output filename prefix]";

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn try_main() -> Result<(), ScanError> {
    let mut args = std::env::args().skip(1);

    let input = match args.next() {
        Some(s) => {
            if matches!(s.as_str(), "-h" | "--help") {
                println!("{}", HELP_MESSAGE);
                return Ok(());
            }
            s
        }
        None => {
            println!("{}", HELP_MESSAGE);
            return Ok(());
        }
    };
    let output = args.next().map_or_else(|| "scan".into(), |s| s);

    let file = std::fs::read(input)?;
    let mut cursor = std::io::Cursor::new(&file);

    let mut buf = [0; 3];

    // Return an error if the file header doesn't match JPEG bytes
    std::io::Read::read_exact(&mut cursor, &mut buf)?;
    if buf != JPEG_MAGIC_BYTES {
        return Err(ScanError::InvalidFile);
    }

    let mut scan_count = 0u16;
    for _ in 0..u32::MAX {
        match find_next_marker(&mut cursor)? {
            SOS => {
                consume_sos_section(&mut cursor)?;

                let scan = std::fs::File::create(format!("{}_{:05}.jpg", output, scan_count))?;
                let mut w = std::io::BufWriter::new(scan);

                let stream_position =
                    std::convert::TryFrom::try_from(std::io::Seek::stream_position(&mut cursor)?)?;

                // Write all data up to the current stream position, followed by an EOI marker
                std::io::Write::write_all(&mut w, &file[..stream_position])?;
                std::io::Write::write_all(&mut w, &[FILL, EOI])?;

                scan_count = scan_count
                    .checked_add(1)
                    .ok_or(ScanError::ScanCounterOverflow)?;
            }
            EOI => break,
            _ => consume_marker_section(&mut cursor)?,
        }
    }

    Ok(())
}

/// Error for processing progressive scan images.
#[derive(Debug)]
pub enum ScanError {
    /// An error occurred during conversion of the stream position to a `usize`.
    Convert(core::num::TryFromIntError),
    /// The supplied file isn't a JPEG.
    InvalidFile,
    /// An error occurred while opening a file, writing to a file, or reading
    /// from a cursor/file.
    Io(std::io::Error),
    /// The scan number counter overflowed.
    ScanCounterOverflow,
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Convert(err) => write!(f, "{}", err),
            Self::InvalidFile => write!(f, "Input file must be a JPEG"),
            Self::Io(err) => write!(f, "{}", err),
            Self::ScanCounterOverflow => write!(f, "Scan counter overflow"),
        }
    }
}

impl std::error::Error for ScanError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Convert(e) => Some(e),
            Self::Io(e) => Some(e),
            Self::InvalidFile | Self::ScanCounterOverflow => None,
        }
    }
}

impl std::convert::From<std::io::Error> for ScanError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl std::convert::From<std::num::TryFromIntError> for ScanError {
    fn from(error: std::num::TryFromIntError) -> Self {
        Self::Convert(error)
    }
}
