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
    let (print_bytes, print_chars, print_newlines, print_words) = {
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

    // Initialize counters
    let mut byte_counter = ByteCounter::new(&metadata);
    let mut char_counter = CharCounter::new();
    let mut newline_counter = NewlineCounter::new();
    let mut word_counter = WordCounter::new();

    // Read file in 16KB chunks
    let mut buffer = [0u8; 16384];
    while let Ok(bytes_read) = reader.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }

        // Byte count
        if print_bytes {
            byte_counter.count_bytes(bytes_read);
        }

        // Char count
        if print_chars {
            char_counter.count_chars(&buffer[..bytes_read]);
        }

        // Line count
        if print_newlines {
            newline_counter.count_newlines(&buffer[..bytes_read]);
        }

        // Word count
        if print_words {
            word_counter.count_words(&buffer[..bytes_read]);
        }
    }

    if char_counter.invalid_chars_found || char_counter.remaining_bytes_in_char != 0 {
        let mut stderr_handle = io::stderr().lock();
        write!(stderr_handle, "Warning: Invalid UTF-8 detected")?;
        if let Some(ref input) = args.input {
            write!(stderr_handle, " in file {}", input.display())?;
        }
        writeln!(stderr_handle)?;
    }

    let output_width = {
        if args.print_bytes as u8 + args.print_chars as u8 + args.print_lines as u8 + args.print_words as u8 == 1 {
            0 // Only one flag, no need for format
        } else if let Some(ref metadata) = metadata {
            (metadata.len().max(1).ilog10() + 1) as usize // byte count digits
        } else {
            7 // stdin, use default 7
        }
    };

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    // In order: newline, word, character, byte, max line length
    if print_newlines {
        write!(handle, "{:>output_width$} ", newline_counter.get())?;
    }
    if print_words {
        write!(handle, "{:>output_width$} ", word_counter.get())?;
    }
    if print_chars {
        write!(handle, "{:>output_width$} ", char_counter.get())?;
    }
    if print_bytes {
        write!(handle, "{:>output_width$} ", byte_counter.get())?;
    }
    if let Some(ref input) = args.input {
        write!(handle, "{}", input.display())?;
    }

    writeln!(handle)?;

    Ok(())
}

struct ByteCounter {
    metadata_found: bool,
    byte_count: u64,
}
impl ByteCounter {
    fn new(metadata: &Option<Metadata>) -> Self {
        match metadata {
            None => Self {
                metadata_found: false,
                byte_count: 0,
            },
            Some(data) => Self {
                metadata_found: true,
                byte_count: data.len(),
            },
        }
    }
    fn count_bytes(&mut self, bytes_read: usize) {
        if !self.metadata_found {
            self.byte_count += bytes_read as u64;
        }
    }
    fn get(&self) -> u64 {
        self.byte_count
    }
}

struct CharCounter {
    char_count: u64,
    remaining_bytes_in_char: usize,
    invalid_chars_found: bool,
}
impl CharCounter {
    fn new() -> Self {
        Self {
            char_count: 0,
            remaining_bytes_in_char: 0,
            invalid_chars_found: false,
        }
    }
    fn count_chars(&mut self, chunk: &[u8]) {
        // Stops counting for invalid characters. Closer to GNU coreutils implementation than uutils.
        for &byte in chunk {
            if byte & (1 << 7) == 0 {
                // 0xxxxxxx, ASCII
                self.char_count += 1;
                self.remaining_bytes_in_char = 0;
            } else if byte & (1 << 6) == 0 {
                // 10xxxxxx, UTF-8 tail
                match self.remaining_bytes_in_char {
                    0 => self.invalid_chars_found = true, // tail with no head
                    _ => {
                        self.remaining_bytes_in_char -= 1;
                        if self.remaining_bytes_in_char == 0 {
                            self.char_count += 1;
                        }
                    }
                }
            } else if byte & (1 << 5) == 0 {
                // 110xxxxx, 2-byte UTF-8 head
                self.remaining_bytes_in_char = 1;
            } else if byte & (1 << 4) == 0 {
                // 1110xxxx, 3-byte UTF-8 head
                self.remaining_bytes_in_char = 2;
            } else if byte & (1 << 3) == 0 {
                // 11110xxx, 4-byte UTF-8 head
                self.remaining_bytes_in_char = 3;
            } else {
                // Invalid byte
                self.invalid_chars_found = true;
                self.remaining_bytes_in_char = 0;
            }
        }
    }
    fn get(&self) -> u64 {
        self.char_count
    }
}

struct NewlineCounter {
    line_count: u64,
}
impl NewlineCounter {
    fn new() -> Self {
        Self { line_count: 0 }
    }
    fn count_newlines(&mut self, chunk: &[u8]) {
        self.line_count += memchr_iter(b'\n', chunk).count() as u64
    }
    fn get(&self) -> u64 {
        self.line_count
    }
}

struct WordCounter {
    word_count: u64,
    in_word: bool,
}
impl WordCounter {
    fn new() -> Self {
        Self {
            word_count: 0,
            in_word: false,
        }
    }
    fn count_words(&mut self, chunk: &[u8]) {
        for &byte in chunk {
            let is_whitespace = byte.is_ascii_whitespace();
            self.word_count += (!is_whitespace && !self.in_word) as u64;
            self.in_word = !is_whitespace;
        }
    }
    fn get(&self) -> u64 {
        self.word_count
    }
}
