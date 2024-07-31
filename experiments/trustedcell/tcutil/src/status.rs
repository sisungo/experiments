pub fn main() -> anyhow::Result<()> {
    match std::fs::read_to_string(crate::trustedcell_securityfs_path().join("status")) {
        Ok(x) if x == "0" => println!("Initialized"),
        Ok(x) if x == "1" => println!("Attached"),
        Ok(_) => println!("Unknown"),
        Err(_) => println!("Uninitialized"),
    };
    Ok(())
}