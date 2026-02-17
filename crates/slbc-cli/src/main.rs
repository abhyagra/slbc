//! SLBC CLI — encode, decode, inspect, transform, roundtrip.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};

use slbc::container;
use slbc::decoder::{self, Script};
use slbc::encoder;
use slbc::inspect;
use slbc::transform;
use slbc::types::*;

#[derive(Parser)]
#[command(name = "slbc", version, about = "Sanskrit Linguistic Binary Codec")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Encode IAST text to .slbc binary
    Encode {
        /// IAST text to encode (if no -i file)
        text: Option<String>,

        /// Input file (IAST text)
        #[arg(short, long)]
        i: Option<PathBuf>,

        /// Output file (.slbc binary)
        #[arg(short, long)]
        o: Option<PathBuf>,

        /// Print hex dump instead of writing binary
        #[arg(long)]
        hex: bool,
    },

    /// Decode .slbc binary to text
    Decode {
        /// Input .slbc file
        #[arg(short, long)]
        i: PathBuf,

        /// Output script: iast or devanagari
        #[arg(long, default_value = "iast")]
        to: String,

        /// Output file (text)
        #[arg(short, long)]
        o: Option<PathBuf>,
    },

    /// Inspect SLBC bytes
    Inspect {
        /// Single byte to inspect (e.g. 0x1B)
        #[arg(long)]
        byte: Option<String>,

        /// Hex stream to inspect (e.g. "1B 40 33 24 40")
        #[arg(long)]
        from_hex: Option<String>,

        /// .slbc file to inspect
        #[arg(short, long)]
        i: Option<PathBuf>,
    },

    /// Apply algebraic transformation to a byte
    Transform {
        /// Operation: guna, vrddhi, dirgha, hrasva, jastva,
        /// toggle-voice, toggle-aspiration, nasal,
        /// samprasarana-svara, samprasarana-sonorant
        #[arg(long)]
        op: String,

        /// Input byte (hex, e.g. 0x44)
        byte: String,

        /// Second byte (for savarṇa-dīrgha)
        byte2: Option<String>,
    },

    /// Round-trip test: encode IAST → .slbc → decode IAST and compare
    Roundtrip {
        /// IAST text to test
        text: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Encode { text, i, o, hex } => cmd_encode(text, i, o, hex),
        Command::Decode { i, to, o } => cmd_decode(i, to, o),
        Command::Inspect { byte, from_hex, i } => cmd_inspect(byte, from_hex, i),
        Command::Transform { op, byte, byte2 } => cmd_transform(op, byte, byte2),
        Command::Roundtrip { text } => cmd_roundtrip(text),
    }
}

// ── Encode ──

fn cmd_encode(text: Option<String>, input: Option<PathBuf>, output: Option<PathBuf>, hex: bool) -> Result<()> {
    let iast = match (text, input) {
        (Some(t), _) => t,
        (None, Some(path)) => fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?,
        (None, None) => bail!("provide IAST text or -i <file>"),
    };

    let iast = iast.trim();
    let phon_payload = encoder::encode_iast(iast)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let slbc_data = container::build_slbc(&phon_payload);

    if hex {
        print_hex(&slbc_data);
        return Ok(());
    }

    match output {
        Some(path) => {
            fs::write(&path, &slbc_data)
                .with_context(|| format!("writing {}", path.display()))?;
            eprintln!("wrote {} bytes to {}", slbc_data.len(), path.display());
        }
        None => {
            print_hex(&slbc_data);
        }
    }

    Ok(())
}

// ── Decode ──

fn cmd_decode(input: PathBuf, to: String, output: Option<PathBuf>) -> Result<()> {
    let data = fs::read(&input)
        .with_context(|| format!("reading {}", input.display()))?;

    let (_header, chunks) = container::parse_slbc(&data)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let script = match to.as_str() {
        "iast" => Script::Iast,
        "devanagari" | "deva" => Script::Devanagari,
        _ => bail!("unknown script '{}' (use 'iast' or 'devanagari')", to),
    };

    let mut full_text = String::new();
    for chunk in &chunks {
        if chunk.chunk_type == CHUNK_PHON {
            let text = decoder::decode_phon(&chunk.payload, script)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            full_text.push_str(&text);
        }
    }

    match output {
        Some(path) => {
            fs::write(&path, &full_text)
                .with_context(|| format!("writing {}", path.display()))?;
            eprintln!("wrote {} chars to {}", full_text.len(), path.display());
        }
        None => {
            println!("{}", full_text);
        }
    }

    Ok(())
}

// ── Inspect ──

fn cmd_inspect(byte: Option<String>, from_hex: Option<String>, input: Option<PathBuf>) -> Result<()> {
    if let Some(byte_str) = byte {
        let b = parse_hex_byte(&byte_str)?;
        let info = inspect::inspect_byte(b);
        println!("{}", inspect::format_byte_info(&info));
        return Ok(());
    }

    if let Some(hex_str) = from_hex {
        let infos = inspect::inspect_hex_stream(&hex_str)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        for (i, info) in infos.iter().enumerate() {
            if i > 0 {
                println!("  ───");
            }
            println!("{}", inspect::format_byte_info(info));
        }
        return Ok(());
    }

    if let Some(path) = input {
        let data = fs::read(&path)
            .with_context(|| format!("reading {}", path.display()))?;

        let (header, chunks) = container::parse_slbc(&data)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        println!("=== SLBC Container ===");
        println!("  Version: {}.{}.{}.{}",
            header.version[0], header.version[1],
            header.version[2], header.version[3]);
        println!("  Flags:   0b{:08b} (0x{:02X})", header.flags, header.flags);
        println!("    HAS_LIPI:     {}", header.has_lipi());
        println!("    HAS_META:     {}", header.has_meta());
        println!("    INTERLEAVED:  {}", header.is_interleaved());
        println!("    VEDIC:        {}", header.is_vedic());
        println!("    VYA:          {}", header.has_vya());
        println!("  Extended header: {} bytes", header.extended_header_len);
        println!("  Chunks: {}", chunks.len());

        for (ci, chunk) in chunks.iter().enumerate() {
            let type_name = match chunk.chunk_type {
                CHUNK_PHON => "PHON",
                CHUNK_BHA => "BHA",
                CHUNK_LIPI => "LIPI",
                CHUNK_META => "META",
                CHUNK_DICT => "DICT",
                CHUNK_IDX => "IDX",
                CHUNK_ANVY => "ANVY",
                CHUNK_EXT => "EXT",
                CHUNK_EOF => "EOF",
                _ => "???",
            };
            println!("\n  Chunk {} — {} (0x{:02X}), {} bytes payload",
                ci, type_name, chunk.chunk_type, chunk.payload.len());

            if chunk.chunk_type == CHUNK_PHON && !chunk.payload.is_empty() {
                println!("    Bytes:");
                for &b in &chunk.payload {
                    let info = inspect::inspect_byte(b);
                    println!("      {:>4}  {}", info.hex, info.description);
                }
            }
        }

        return Ok(());
    }

    bail!("provide --byte, --from-hex, or -i <file>");
}

// ── Transform ──

fn cmd_transform(op: String, byte_str: String, byte2_str: Option<String>) -> Result<()> {
    let b = parse_hex_byte(&byte_str)?;

    let result = match op.as_str() {
        "guna" => transform::guna(b),
        "vrddhi" => transform::vrddhi(b),
        "dirgha" => transform::dirgha(b),
        "hrasva" => transform::hrasva(b),
        "jastva" => transform::jastva(b),
        "toggle-voice" => transform::toggle_voice(b),
        "toggle-aspiration" => transform::toggle_aspiration(b),
        "nasal" => transform::make_nasal(b),
        "homorganic-nasal" => transform::homorganic_nasal(b),
        "samprasarana-svara" => transform::samprasarana_to_svara(b),
        "samprasarana-sonorant" => transform::samprasarana_to_sonorant(b),
        "savarna-dirgha" => {
            let b2_str = byte2_str.ok_or_else(|| anyhow::anyhow!("savarṇa-dīrgha requires two bytes"))?;
            let b2 = parse_hex_byte(&b2_str)?;
            transform::savarna_dirgha(b, b2)
        }
        _ => bail!("unknown operation '{}'\nValid: guna, vrddhi, dirgha, hrasva, jastva, toggle-voice, toggle-aspiration, nasal, homorganic-nasal, samprasarana-svara, samprasarana-sonorant, savarna-dirgha", op),
    }
    .map_err(|e| anyhow::anyhow!("{}", e))?;

    println!(
        "{}: {} (0x{:02X}) → {} (0x{:02X})",
        result.operation,
        result.input_iast, result.input_byte,
        result.output_iast, result.output_byte
    );

    Ok(())
}

// ── Roundtrip ──

fn cmd_roundtrip(text: String) -> Result<()> {
    let input = text.trim();
    eprintln!("Input (IAST):  {}", input);

    // Encode
    let phon_payload = encoder::encode_iast(input)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let slbc_data = container::build_slbc(&phon_payload);

    eprintln!("Encoded:       {} bytes (.slbc container)", slbc_data.len());
    eprint!("PHON payload:  ");
    for b in &phon_payload {
        eprint!("{:02X} ", b);
    }
    eprintln!();

    // Decode back to IAST
    let (_, chunks) = container::parse_slbc(&slbc_data)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let mut decoded = String::new();
    for chunk in &chunks {
        if chunk.chunk_type == CHUNK_PHON {
            let text = decoder::decode_phon(&chunk.payload, Script::Iast)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            decoded.push_str(&text);
        }
    }

    eprintln!("Output (IAST): {}", decoded);

    // Also show Devanāgarī
    let mut deva = String::new();
    for chunk in &chunks {
        if chunk.chunk_type == CHUNK_PHON {
            let text = decoder::decode_phon(&chunk.payload, Script::Devanagari)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            deva.push_str(&text);
        }
    }
    eprintln!("Output (Deva): {}", deva);

    if decoded == input {
        eprintln!("\n✓ Round-trip PASSED");
    } else {
        eprintln!("\n✗ Round-trip FAILED");
        eprintln!("  expected: {:?}", input);
        eprintln!("  got:      {:?}", decoded);
        std::process::exit(1);
    }

    Ok(())
}

// ── Helpers ──

fn parse_hex_byte(s: &str) -> Result<u8> {
    let s = s.trim().trim_start_matches("0x").trim_start_matches("0X");
    u8::from_str_radix(s, 16)
        .with_context(|| format!("invalid hex byte: '{}'", s))
}

fn print_hex(data: &[u8]) {
    for (i, chunk) in data.chunks(16).enumerate() {
        print!("{:08X}  ", i * 16);
        for (j, b) in chunk.iter().enumerate() {
            if j == 8 {
                print!(" ");
            }
            print!("{:02X} ", b);
        }
        // Pad if short row
        let pad = 16 - chunk.len();
        for _ in 0..pad {
            print!("   ");
        }
        if chunk.len() <= 8 {
            print!(" ");
        }
        print!(" |");
        for &b in chunk {
            let c = if b >= 0x20 && b < 0x7F { b as char } else { '.' };
            print!("{}", c);
        }
        println!("|");
    }
}
