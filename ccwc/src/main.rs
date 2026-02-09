use std::{
    error::Error,
    fs::{self, File, Metadata},
    io::{self, Read, Write},
    path::PathBuf,
};

use clap::Parser;
use memchr::memchr_iter;

#[derive(Parser, Debug)]
struct Args {
    input: Option<PathBuf>,

    #[arg(short = 'c', long = "bytes")]
    print_bytes: bool,

    #[arg(short = 'm', long = "chars")]
    print_chars: bool,

    #[arg(short = 'l', long = "lines")]
    print_lines: bool,

    #[arg(short = 'w', long = "words")]
    print_words: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Set all flags to true if none are selected
    let (print_bytes, print_chars, print_lines, print_words) = {
        if !(args.print_bytes || args.print_chars || args.print_lines || args.print_words) {
            (true, false, true, true)
        } else {
            (args.print_bytes, args.print_chars, args.print_lines, args.print_words)
        }
    };

    let (mut reader, metadata): (Box<dyn Read>, Option<Metadata>) = match &args.input {
        None => {
            // Read from stdin
            (Box::new(io::stdin()), None)
        }
        Some(path) => {
            // Read from path
            (Box::new(File::open(path)?), fs::metadata(path).ok())
        }
    };

    // If metadata exists, use it to find byte count and output width. If not, use default values
    let (mut bytes, output_width) = if let Some(ref meta) = metadata {
        (meta.len(), (meta.len().max(1).ilog10() + 1) as usize)
    } else {
        (0, 7)
    };

    // Initialize counters
    let (mut chars, mut lines, mut words) = (0, 0, 0);
    let mut in_word = false;

    // Read file in 8KB chunks
    let mut buffer = [0u8; 8192];
    while let Ok(bytes_read) = reader.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }
        let chunk = &buffer[..bytes_read];

        // Byte count
        if print_bytes && metadata.is_none() {
            bytes += bytes_read as u64;
        }

        // Char count

        // Line count
        if print_lines {
            lines += memchr_iter(b'\n', chunk).count() as u64;
        }

        // Word count
        if print_words {
            for &byte in chunk {
                let is_whitespace = byte == b' ' || (byte.wrapping_sub(9) <= 4); // 9-13: \t, \n, \v, \f, \r
                words += (!is_whitespace && !in_word) as u64;
                in_word = !is_whitespace;
            }
        }
    }

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    if print_lines {
        write!(handle, "{:>output_width$} ", lines)?;
    }
    if print_words {
        write!(handle, "{:>output_width$} ", words)?;
    }
    if print_bytes {
        write!(handle, "{:>output_width$} ", bytes)?;
    }
    if let Some(input) = args.input {
        write!(handle, "{}", input.display())?;
    }

    writeln!(handle)?;

    Ok(())
}
