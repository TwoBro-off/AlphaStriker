use serde::Deserialize;
use std::collections::HashMap;

const INITIALIZE2_DISCRIMINANT: [u8; 8] = [0xd8, 0x1c, 0x8e, 0x23, 0x84, 0x96, 0xe9, 0x9b];
const RAYDIUM_LIQUIDITY_POOL_V4: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

#[derive(Debug, Deserialize)]
struct Instruction {
    #[serde(rename = "programId")]
    program_id: String,
    data: String,
}

#[derive(Debug, Deserialize)]
struct Message {
    #[serde(rename = "accountKeys")]
    account_keys: Vec<String>,
    instructions: Vec<Instruction>,
}

#[derive(Debug, Deserialize)]
struct Transaction {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct TransactionResult {
    transaction: Transaction,
}

#[derive(Debug, Deserialize)]
struct GetTransactionResponse {
    result: Option<TransactionResult>,
}

pub fn extract_pool_keys_from_tx_internal(json_data: &str) -> Option<HashMap<String, String>> {
    let response: GetTransactionResponse = match serde_json::from_str(json_data) {
        Ok(res) => res,
        Err(_) => return None,
    };

    if let Some(tx_result) = response.result {
        for ix in &tx_result.transaction.message.instructions {
            if ix.program_id != RAYDIUM_LIQUIDITY_POOL_V4 {
                continue;
            }

            let ix_data = match bs58::decode(&ix.data).into_vec() {
                Ok(data) => data,
                Err(_) => continue,
            };

            if ix_data.len() >= 8 && compare_discriminant_optimized(&ix_data[..8]) {
                let accounts = &tx_result.transaction.message.account_keys;
                if accounts.len() > 16 {
                    let mut result = HashMap::new();
                    result.insert("amm_id".to_string(), accounts.get(4).cloned().unwrap_or_default());
                    result.insert("base_mint".to_string(), accounts.get(8).cloned().unwrap_or_default());
                    result.insert("quote_mint".to_string(), accounts.get(9).cloned().unwrap_or_default());
                    return Some(result);
                }
            }
        }
    }
    None
}

#[inline(always)]
fn compare_discriminant_optimized(data: &[u8]) -> bool {
    #[cfg(all(target_arch = "aarch64", feature = "neon_opt"))]
    {
        if std::is_aarch64_feature_detected!("neon") {
            return unsafe { compare_discriminant_neon(data) };
        }
    }

    #[cfg(target_arch = "x86_64")]
    {
        if std::is_x86_feature_detected!("avx2") {
            return unsafe { compare_discriminant_avx2(data) };
        }
    }

    compare_discriminant_fallback(data)
}

#[cfg(all(target_arch = "aarch64", feature = "neon_opt"))]
#[target_feature(enable = "neon")]
#[inline]
unsafe fn compare_discriminant_neon(data: &[u8]) -> bool {
    // SAFETY: 
    // 1. L'appelant doit garantir que `data` est une slice d'au moins 8 octets.
    //    Ceci est assuré par `ix_data.len() >= 8` dans `extract_pool_keys_from_tx_internal`.
    // 2. Les pointeurs `data.as_ptr()` et `INITIALIZE2_DISCRIMINANT.as_ptr()` sont valides.
    // 3. Les options `nostack`, `pure`, `nomem` sont correctes car cette fonction est un calcul pur sans effets de bord.
    use std::arch::aarch64::*;
    use std::arch::asm;

    let result: u8;

    asm!(
        "ld1 {{v0.8b}}, [{data_ptr}]",
        "ld1 {{v1.8b}}, [{discriminant_ptr}]",
        "cmeq v0.8b, v0.8b, v1.8b",
        "uminv b0, v0.8b",
        "umov {w_out}, v0.b[0]",
        data_ptr = in(reg) data.as_ptr(),
        discriminant_ptr = in(reg) INITIALIZE2_DISCRIMINANT.as_ptr(),
        w_out = out(reg) result,
        out("v0") _, out("v1") _,
        options(nostack, pure, nomem)
    );

    result == 0xFF
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx", enable = "avx2")]
#[inline]
unsafe fn compare_discriminant_avx2(data: &[u8]) -> bool {
    // SAFETY: 
    // 1. L'appelant doit garantir que `data` est une slice d'au moins 8 octets.
    //    Ceci est assuré par `ix_data.len() >= 8` dans `extract_pool_keys_from_tx_internal`.
    // 2. Les pointeurs `data.as_ptr()` et `INITIALIZE2_DISCRIMINANT.as_ptr()` sont valides.
    // 3. Les options `nostack`, `pure`, `nomem` sont correctes car cette fonction est un calcul pur sans effets de bord.
    use std::arch::asm;
    let result: u32;

    asm!(
        "vmovq {xmm_data}, [{data_ptr}]",
        "vmovq {xmm_discriminant}, [{discriminant_ptr}]",
        "vpcmpeqb {xmm_data}, {xmm_data}, {xmm_discriminant}",
        "vpmovmskb {xmm_data}, {gp_out:e}",
        data_ptr = in(reg) data.as_ptr(),
        discriminant_ptr = in(reg) INITIALIZE2_DISCRIMINANT.as_ptr(),
        xmm_data = out(xmm_reg) _,
        xmm_discriminant = out(xmm_reg) _,
        gp_out = out(reg) result,
        options(nostack, pure, nomem, att_syntax)
    );

    result & 0xFF == 0xFF
}

#[inline]
fn compare_discriminant_fallback(data: &[u8]) -> bool {
    data == &INITIALIZE2_DISCRIMINANT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_fallback() {
        assert!(compare_discriminant_fallback(&INITIALIZE2_DISCRIMINANT));
        let wrong_data = [0, 1, 2, 3, 4, 5, 6, 7];
        assert!(!compare_discriminant_fallback(&wrong_data));
    }

    #[test]
    #[cfg(all(target_arch = "aarch64", feature = "neon_opt"))]
    fn test_compare_neon() {
        assert!(unsafe { compare_discriminant_neon(&INITIALIZE2_DISCRIMINANT) });
        let wrong_data = [0, 1, 2, 3, 4, 5, 6, 7];
        assert!(!unsafe { compare_discriminant_neon(&wrong_data) });
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_compare_avx2() {
        if !std::is_x86_feature_detected!("avx2") {
            return;
        }
        assert!(unsafe { compare_discriminant_avx2(&INITIALIZE2_DISCRIMINANT) });
        let wrong_data = [0, 1, 2, 3, 4, 5, 6, 7];
        assert!(!unsafe { compare_discriminant_avx2(&wrong_data) });
    }
}