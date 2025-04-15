use anyhow::{anyhow, Result};
use btclib::crypto::PublicKey;
use btclib::network::Message;
use btclib::util::Saveable;
use std::env;
use std::process::exit;
use tokio::net::TcpStream;

fn usage() -> ! {
    eprintln!(
        "Usage: {} <address> <public_key_file>",
        env::args().next().unwrap()
    );
    exit(1);
}

#[tokio::main]
async fn main() -> Result<()> {
    let address = match env::args().nth(1) {
        Some(address) => address,
        None => usage(),
    };
    let public_key_file = match env::args().nth(2) {
        Some(public_key_file) => public_key_file,
        None => usage(),
    };
    let public_key = PublicKey::load_from_file(&public_key_file)
        .map_err(|e| anyhow!("Error reading publickey: {}", e))?;
    todo!()
}
/*
   let (path, steps) = if let (Some(_1), Some(_2)) = (env::args().nth(1), env::args().nth(2)) {
       (_1, _2)
   } else {
       eprintln!("Usage: miner <block-file> <steps>");
       exit(1);
   };
   let steps: usize = if let Ok(s @ 1..usize::MAX) = steps.parse() {
       s
   } else {
       eprintln!("<steps> should be a positive integer");
       exit(1);
   };

   let og_block = Block::load_from_file(path).expect("failed to load block");
   let mut block = og_block.clone();
   while !block.header.mine(steps) {
       println!("mining")
   }
   println!("original: {:#?}", og_block);
   println!("hash: {}", og_block.header.hash());
   // print mined block and its hash
   println!("final: {:#?}", block);
   println!("hash: {}", block.header.hash());
*/
