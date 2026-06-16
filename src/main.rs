use clap::{Parser, Subcommand};
use serde_json::json;
use soroban_toolkit::{address, encoding, hash, transaction};
use std::process;

#[derive(Parser)]
#[command(
    name = "soroban-toolkit",
    version,
    about = "Soroban/Stellar developer toolkit"
)]
struct Cli {
    #[arg(long, global = true, help = "Output results as JSON")]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Address utilities
    Address {
        #[command(subcommand)]
        cmd: AddressCmd,
    },
    /// Hashing utilities
    Hash {
        #[command(subcommand)]
        cmd: HashCmd,
    },
    /// Encoding utilities
    Encode {
        #[command(subcommand)]
        cmd: EncodeCmd,
    },
    /// Transaction utilities
    Tx {
        #[command(subcommand)]
        cmd: TxCmd,
    },
}

#[derive(Subcommand)]
enum AddressCmd {
    /// Validate a Stellar/Soroban address
    Validate { address: String },
    /// Mask an address (show first 4 and last 4 chars)
    Mask { address: String },
    /// Detect the type of a Stellar address
    DetectType { address: String },
}

#[derive(Subcommand)]
enum HashCmd {
    /// Compute SHA-256 hex digest
    Sha256 { input: String },
    /// Compute SHA-512 hex digest
    Sha512 { input: String },
    /// Compute double-SHA-256 hex digest
    DoubleSha256 { input: String },
}

#[derive(Subcommand)]
enum EncodeCmd {
    /// Encode input string to hex
    ToHex { input: String },
    /// Decode hex string to UTF-8
    FromHex { hex: String },
    /// Encode input string to base64
    ToBase64 { input: String },
    /// Decode base64 string to UTF-8
    FromBase64 { b64: String },
}

#[derive(Subcommand)]
enum TxCmd {
    /// Format stroops as XLM string
    FormatXlm { stroops: u64 },
    /// Validate a transaction hash
    ValidateHash { hash: String },
    /// Normalize a transaction hash (lowercase, strip 0x)
    NormalizeHash { hash: String },
    /// Estimate transaction fee
    EstimateFee { base_fee: u32, ops: u32 },
}

struct Printer {
    json: bool,
}

impl Printer {
    fn success(&self, data: serde_json::Value) {
        if self.json {
            println!("{}", json!({"success": true, "data": data}));
        } else {
            match &data {
                serde_json::Value::String(s) => println!("{s}"),
                other => println!("{other}"),
            }
        }
    }

    fn error(&self, msg: &str) -> ! {
        if self.json {
            println!("{}", json!({"success": false, "error": msg}));
        } else {
            eprintln!("Error: {msg}");
        }
        process::exit(1);
    }
}

fn main() {
    let cli = Cli::parse();
    let printer = Printer { json: cli.json };

    match cli.command {
        Commands::Address { cmd } => match cmd {
            AddressCmd::Validate { address: addr } => match address::validate_address(&addr) {
                Ok(_) => printer.success(json!({
                    "address": addr,
                    "valid": true,
                    "type": format!("{:?}", address::detect_address_type(&addr))
                })),
                Err(e) => printer.error(&e.to_string()),
            },
            AddressCmd::Mask { address: addr } => {
                printer.success(json!(address::mask_address(&addr)));
            }
            AddressCmd::DetectType { address: addr } => {
                let kind = address::detect_address_type(&addr);
                printer.success(json!(format!("{kind:?}")));
            }
        },

        Commands::Hash { cmd } => match cmd {
            HashCmd::Sha256 { input } => {
                printer.success(json!(hash::sha256_hex(input.as_bytes())));
            }
            HashCmd::Sha512 { input } => {
                printer.success(json!(hash::sha512_hex(input.as_bytes())));
            }
            HashCmd::DoubleSha256 { input } => {
                printer.success(json!(hash::double_sha256(input.as_bytes())));
            }
        },

        Commands::Encode { cmd } => match cmd {
            EncodeCmd::ToHex { input } => {
                printer.success(json!(encoding::to_hex(input.as_bytes())));
            }
            EncodeCmd::FromHex { hex } => match encoding::from_hex(&hex) {
                Ok(bytes) => match String::from_utf8(bytes) {
                    Ok(s) => printer.success(json!(s)),
                    Err(_) => printer.error("Decoded bytes are not valid UTF-8"),
                },
                Err(e) => printer.error(&e.to_string()),
            },
            EncodeCmd::ToBase64 { input } => {
                printer.success(json!(encoding::to_base64(input.as_bytes())));
            }
            EncodeCmd::FromBase64 { b64 } => match encoding::from_base64(&b64) {
                Ok(bytes) => match String::from_utf8(bytes) {
                    Ok(s) => printer.success(json!(s)),
                    Err(_) => printer.error("Decoded bytes are not valid UTF-8"),
                },
                Err(e) => printer.error(&e.to_string()),
            },
        },

        Commands::Tx { cmd } => match cmd {
            TxCmd::FormatXlm { stroops } => {
                printer.success(json!(transaction::format_xlm(stroops)));
            }
            TxCmd::ValidateHash { hash: h } => {
                let valid = transaction::is_valid_tx_hash(&h);
                if !valid {
                    printer.error("invalid transaction hash");
                }
                if cli.json {
                    printer.success(json!({"hash": h, "valid": true}));
                } else {
                    println!("valid");
                }
            }
            TxCmd::NormalizeHash { hash: h } => match transaction::normalize_tx_hash(&h) {
                Ok(normalized) => printer.success(json!(normalized)),
                Err(e) => printer.error(&e.to_string()),
            },
            TxCmd::EstimateFee { base_fee, ops } => {
                let stroops = transaction::estimate_fee(base_fee, ops);
                let xlm = transaction::estimate_fee_xlm(base_fee, ops);
                printer.success(json!({
                    "stroops": stroops,
                    "xlm": transaction::format_xlm(stroops as u64)
                }));
                let _ = xlm; // used via format_xlm above
            }
        },
    }
}
