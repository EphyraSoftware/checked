/// Decode a MiniSign verification key from its text representation.
pub fn try_read_mini_sign_vf_key(input: &str) -> anyhow::Result<minisign_verify::PublicKey> {
    Ok(minisign_verify::PublicKey::decode(input)?)
}

/// Check that the proof and proof_sig provided with a [VerificationKeyDist] are valid with respect to
/// the verification key.
pub fn check_mini_sign_proof(
    vf_key: &minisign_verify::PublicKey,
    proof: &[u8],
    proof_sig: Vec<u8>,
) -> anyhow::Result<()> {
    // In general, treat proof signatures as opaque bytes. For MiniSign, we can decode them to strings.
    let sig = String::from_utf8(proof_sig)
        .map_err(|e| anyhow::anyhow!("Signature is not valid UTF-8: {}", e))?;
    let sig = minisign_verify::Signature::decode(&sig)
        .map_err(|e| anyhow::anyhow!("Failed to decode signature: {}", e))?;

    Ok(vf_key.verify(proof, &sig, false)?)
}
