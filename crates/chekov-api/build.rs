fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().out_dir("src/generated").compile(
        &["proto/chekov.proto", "proto/chekov.v2.proto"],
        &["proto/"],
    )?;
    println!("BUILDING");
    Ok(())
}
