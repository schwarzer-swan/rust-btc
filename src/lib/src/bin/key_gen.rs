use std::env;

use btclib::crypto::PrivateKey;
use btclib::util::Saveable;

fn main() {
    let name = env::args().nth(1).expect("Please provide a name");

    let private_key = PrivateKey::new_key();
    let public_key = private_key.public_key();
}
