pub fn serialize<S>(
    key: &super::SigningKey<super::Secp256k1>,
    serializer: S) -> Result<S:: Ok, S::Error>
where
    s: serde::Serialize,
{
    serializer.serialize_bytes(&key.to_bytes())
}
