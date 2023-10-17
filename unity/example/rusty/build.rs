pub fn main() -> anyhow::Result<()> {
    bevity::build("src/exported.rs", "../Assets/bevity")?;

    Ok(())
}
