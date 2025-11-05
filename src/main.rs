#![feature(portable_simd)]
// use memmap::MmapOptions;
use std::io::{self, Read};
use std::{fs, io::stdout, path::PathBuf};

use anyhow::Result;
use clap::{Parser, builder::styling};
use std::simd::Simd;
use std::simd::cmp::SimdPartialEq;

use multi_reader::MultiReader;

// 1MB of buffer size speeds up the program massively, by 2-3x with respect to 8KB for both
// reading from file and from stdin. I guess it's because it allows to reduce the calls to
// writer.write_all, to reduce the time spent processing remainders.
const BUF_SIZE: usize = 1024 * 1024; // 1048576 B
const LANES: usize = 64; // u8x64
const DEFAULT_LINE_FEED: &str = "\n";

const STYLES: styling::Styles = styling::Styles::styled()
    .header(styling::AnsiColor::Green.on_default().bold())
    .usage(styling::AnsiColor::Green.on_default().bold())
    .literal(styling::AnsiColor::Blue.on_default().bold())
    .placeholder(styling::AnsiColor::Cyan.on_default());

/// Split a string based on a delimiter, effectively replacing it with a newline.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(name = "splitl")]
#[command(styles=STYLES)]
struct Cli {
    /// Files to count. If nothing is passed data will be read from STDIN
    inputs: Vec<PathBuf>,

    /// Delimiter to use to split the lines
    #[arg(short, long, default_value = " ", value_parser=parse_char)]
    delimiter: u8,

    /// Line separator
    #[arg(short, long, default_value = DEFAULT_LINE_FEED, value_parser=parse_char)]
    line_sep: u8,
}

fn parse_char(s: &str) -> Result<u8, &'static str> {
    match s {
        "\\t" => Ok(b'\t'),
        "\\n" => Ok(b'\n'),
        "\\r" => Ok(b'\r'),
        s if s.bytes().count() == 1 => Ok(s.bytes().next().unwrap()),
        _ => Err("Delimiter must be a single character"),
    }
}

/// Efficiently reads from `reader`, replaces every `from` byte with `to`,
/// and writes the result to `writer` without any intermediate copies.
///
/// The function uses a local 1 MiB buffer and processes the buffer in
/// SIMD-sized chunks to minimise per-byte overhead.
pub fn substitute<R, W>(mut reader: R, mut writer: W, from: u8, to: u8) -> io::Result<()>
where
    R: io::Read,
    W: io::Write,
{
    let mut buf = vec![0; BUF_SIZE];

    // SIMD masks used inside the hot loop
    let from_vec = Simd::<u8, LANES>::splat(from);
    let to_vec = Simd::<u8, LANES>::splat(to);

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }

        // Operate on the filled portion of the buffer
        let data = &mut buf[..n];

        // Process in 64-byte SIMD chunks
        for chunk in data.chunks_exact_mut(LANES) {
            let mut v = Simd::<u8, LANES>::from_slice(chunk);
            let mask = v.simd_eq(from_vec);
            v = mask.select(to_vec, v);
            chunk.copy_from_slice(&v.to_array());
        }

        // Handle any trailing bytes that didn't fit into a full SIMD chunk
        let remainder = data.chunks_exact_mut(LANES).into_remainder();
        for byte in remainder {
            if *byte == from {
                *byte = to;
            }
        }

        writer.write_all(data)?;
    }

    writer.flush()
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle multiple readers. The output will all be concatenated
    let reader: Box<dyn Read> = if cli.inputs.is_empty() {
        Box::new(io::BufReader::with_capacity(BUF_SIZE, io::stdin()))
    } else {
        let mut handles = vec![];
        for file in cli.inputs.iter() {
            match fs::File::open(file) {
                Ok(handle) => {
                    let rdr = io::BufReader::with_capacity(BUF_SIZE, handle);
                    handles.push(rdr)
                }
                Err(_) => eprintln!("Unable to open file {:?}", file),
            }
        }
        let reader = MultiReader::new(handles.into_iter());
        Box::new(reader)
    };

    let writer = stdout().lock();
    substitute(reader, writer, cli.delimiter, cli.line_sep)?;

    Ok(())
}
