use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_soroban-toolkit");
const VALID_ACCOUNT: &str = "GCEZWKCA5VLDNRLN3RPRJMRZOX3Z6G5CHCGZN36UWBE5XFGT35JA5UMG";
const VALID_CONTRACT: &str = "CCEZWKCA5VLDNRLN3RPRJMRZOX3Z6G5CHCGZN36UWBE5XFGT35JA5UMG";
const VALID_TX_HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn run(args: &[&str]) -> (String, String, i32) {
    let output = Command::new(BIN)
        .args(args)
        .output()
        .expect("failed to run binary");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let code = output.status.code().unwrap_or(-1);
    (stdout, stderr, code)
}

fn json_val(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim()).expect("stdout is not valid JSON")
}

// ── address validate ──────────────────────────────────────────────────────────

#[test]
fn address_validate_text_valid() {
    let (stdout, stderr, code) = run(&["address", "validate", VALID_ACCOUNT]);
    assert_eq!(code, 0);
    assert!(stderr.is_empty());
    assert!(stdout.contains(VALID_ACCOUNT));
}

#[test]
fn address_validate_json_flag_before_subcommand() {
    let (stdout, _stderr, code) = run(&["--json", "address", "validate", VALID_ACCOUNT]);
    assert_eq!(code, 0);
    let v = json_val(&stdout);
    assert_eq!(v["success"], true);
    assert_eq!(v["data"]["valid"], true);
}

#[test]
fn address_validate_json_flag_after_subcommand() {
    // clap global flag — position after subcommand should also work
    let (stdout, _stderr, code) = run(&["address", "--json", "validate", VALID_ACCOUNT]);
    assert_eq!(code, 0);
    let v = json_val(&stdout);
    assert_eq!(v["success"], true);
}

#[test]
fn address_validate_json_invalid_address() {
    let (stdout, _stderr, code) = run(&["--json", "address", "validate", "BADINPUT"]);
    assert_eq!(code, 1);
    let v = json_val(&stdout);
    assert_eq!(v["success"], false);
    assert!(v["error"].as_str().is_some());
}

#[test]
fn address_validate_text_invalid_exits_nonzero() {
    let (_stdout, stderr, code) = run(&["address", "validate", "BADINPUT"]);
    assert_eq!(code, 1);
    assert!(stderr.contains("Error:"));
}

#[test]
fn address_mask_json() {
    let (stdout, _stderr, code) = run(&["--json", "address", "mask", VALID_ACCOUNT]);
    assert_eq!(code, 0);
    let v = json_val(&stdout);
    assert_eq!(v["success"], true);
    assert_eq!(v["data"], "GCEZ...5UMG");
}

#[test]
fn address_mask_text() {
    let (stdout, _stderr, code) = run(&["address", "mask", VALID_ACCOUNT]);
    assert_eq!(code, 0);
    assert!(stdout.trim() == "GCEZ...5UMG");
}

#[test]
fn address_detect_type_account_json() {
    let (stdout, _stderr, code) = run(&["--json", "address", "detect-type", VALID_ACCOUNT]);
    assert_eq!(code, 0);
    let v = json_val(&stdout);
    assert_eq!(v["success"], true);
    assert_eq!(v["data"], "Account");
}

#[test]
fn address_detect_type_contract_json() {
    let (stdout, _stderr, code) = run(&["--json", "address", "detect-type", VALID_CONTRACT]);
    assert_eq!(code, 0);
    let v = json_val(&stdout);
    assert_eq!(v["data"], "Contract");
}

#[test]
fn address_detect_type_invalid_json() {
    let (stdout, _stderr, code) = run(&["--json", "address", "detect-type", "NOPE"]);
    assert_eq!(code, 0);
    let v = json_val(&stdout);
    assert_eq!(v["data"], "Invalid");
}

// ── hash ──────────────────────────────────────────────────────────────────────

#[test]
fn hash_sha256_json() {
    let (stdout, _stderr, code) = run(&["--json", "hash", "sha256", "hello"]);
    assert_eq!(code, 0);
    let v = json_val(&stdout);
    assert_eq!(v["success"], true);
    assert_eq!(
        v["data"],
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
}

#[test]
fn hash_sha256_text() {
    let (stdout, _stderr, code) = run(&["hash", "sha256", "hello"]);
    assert_eq!(code, 0);
    assert!(stdout
        .trim()
        .contains("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"));
}

#[test]
fn hash_sha512_json() {
    let (stdout, _stderr, code) = run(&["--json", "hash", "sha512", "hello"]);
    assert_eq!(code, 0);
    let v = json_val(&stdout);
    assert_eq!(v["success"], true);
    let digest = v["data"].as_str().unwrap();
    assert_eq!(digest.len(), 128);
}

#[test]
fn hash_double_sha256_json() {
    let (stdout, _stderr, code) = run(&["--json", "hash", "double-sha256", "hello"]);
    assert_eq!(code, 0);
    let v = json_val(&stdout);
    assert_eq!(v["success"], true);
    assert_eq!(v["data"].as_str().unwrap().len(), 64);
}

// ── encode ────────────────────────────────────────────────────────────────────

#[test]
fn encode_to_hex_json() {
    let (stdout, _stderr, code) = run(&["--json", "encode", "to-hex", "hello"]);
    assert_eq!(code, 0);
    let v = json_val(&stdout);
    assert_eq!(v["success"], true);
    assert_eq!(v["data"], "68656c6c6f");
}

#[test]
fn encode_from_hex_json() {
    let (stdout, _stderr, code) = run(&["--json", "encode", "from-hex", "68656c6c6f"]);
    assert_eq!(code, 0);
    let v = json_val(&stdout);
    assert_eq!(v["success"], true);
    assert_eq!(v["data"], "hello");
}

#[test]
fn encode_from_hex_invalid_json() {
    let (stdout, _stderr, code) = run(&["--json", "encode", "from-hex", "ZZZZ"]);
    assert_eq!(code, 1);
    let v = json_val(&stdout);
    assert_eq!(v["success"], false);
}

#[test]
fn encode_to_base64_json() {
    let (stdout, _stderr, code) = run(&["--json", "encode", "to-base64", "hello"]);
    assert_eq!(code, 0);
    let v = json_val(&stdout);
    assert_eq!(v["success"], true);
    assert_eq!(v["data"], "aGVsbG8=");
}

#[test]
fn encode_from_base64_json() {
    let (stdout, _stderr, code) = run(&["--json", "encode", "from-base64", "aGVsbG8="]);
    assert_eq!(code, 0);
    let v = json_val(&stdout);
    assert_eq!(v["success"], true);
    assert_eq!(v["data"], "hello");
}

#[test]
fn encode_from_base64_invalid_json() {
    let (stdout, _stderr, code) = run(&["--json", "encode", "from-base64", "!!!!"]);
    assert_eq!(code, 1);
    let v = json_val(&stdout);
    assert_eq!(v["success"], false);
}

#[test]
fn encode_hex_roundtrip_text() {
    let (hex_out, _, _) = run(&["encode", "to-hex", "soroban"]);
    let hex = hex_out.trim();
    let (decoded, _, code) = run(&["encode", "from-hex", hex]);
    assert_eq!(code, 0);
    assert_eq!(decoded.trim(), "soroban");
}

// ── tx ────────────────────────────────────────────────────────────────────────

#[test]
fn tx_format_xlm_json() {
    let (stdout, _stderr, code) = run(&["--json", "tx", "format-xlm", "10000000"]);
    assert_eq!(code, 0);
    let v = json_val(&stdout);
    assert_eq!(v["success"], true);
    assert_eq!(v["data"], "1.0000000 XLM");
}

#[test]
fn tx_format_xlm_text() {
    let (stdout, _stderr, code) = run(&["tx", "format-xlm", "10000000"]);
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "1.0000000 XLM");
}

#[test]
fn tx_validate_hash_valid_json() {
    let (stdout, _stderr, code) = run(&["--json", "tx", "validate-hash", VALID_TX_HASH]);
    assert_eq!(code, 0);
    let v = json_val(&stdout);
    assert_eq!(v["success"], true);
    assert_eq!(v["data"]["valid"], true);
}

#[test]
fn tx_validate_hash_invalid_json() {
    let (stdout, _stderr, code) = run(&["--json", "tx", "validate-hash", "short"]);
    assert_eq!(code, 1);
    let v = json_val(&stdout);
    assert_eq!(v["success"], false);
}

#[test]
fn tx_validate_hash_valid_text() {
    let (stdout, _stderr, code) = run(&["tx", "validate-hash", VALID_TX_HASH]);
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "valid");
}

#[test]
fn tx_normalize_hash_json() {
    let upper = VALID_TX_HASH.to_uppercase();
    let (stdout, _stderr, code) = run(&["--json", "tx", "normalize-hash", &upper]);
    assert_eq!(code, 0);
    let v = json_val(&stdout);
    assert_eq!(v["success"], true);
    assert_eq!(v["data"], VALID_TX_HASH);
}

#[test]
fn tx_normalize_hash_strips_0x_json() {
    let with_prefix = format!("0x{VALID_TX_HASH}");
    let (stdout, _stderr, code) = run(&["--json", "tx", "normalize-hash", &with_prefix]);
    assert_eq!(code, 0);
    let v = json_val(&stdout);
    assert_eq!(v["data"], VALID_TX_HASH);
}

#[test]
fn tx_normalize_hash_invalid_json() {
    let (stdout, _stderr, code) = run(&["--json", "tx", "normalize-hash", "not-a-hash"]);
    assert_eq!(code, 1);
    let v = json_val(&stdout);
    assert_eq!(v["success"], false);
}

#[test]
fn tx_estimate_fee_json() {
    let (stdout, _stderr, code) = run(&["--json", "tx", "estimate-fee", "100", "3"]);
    assert_eq!(code, 0);
    let v = json_val(&stdout);
    assert_eq!(v["success"], true);
    assert_eq!(v["data"]["stroops"], 300);
    assert!(v["data"]["xlm"].as_str().unwrap().contains("XLM"));
}

#[test]
fn tx_estimate_fee_text() {
    let (stdout, _stderr, code) = run(&["tx", "estimate-fee", "100", "3"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("XLM"));
}
