use std::env;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key = "PROTOC";
   // env::set_var(key, "C:/Users/shoba/protoc/bin/protoc.exe");

    tonic_build::compile_protos("proto/orderbook.proto")?;
    Ok(())
}