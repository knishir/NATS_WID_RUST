//Generate Operator, Account, and User from Rust

use nkeys::KeyPair;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create Operator key
    let operator = KeyPair::new_operator();
    let operator_pub = operator.public_key();
    let operator_seed = operator.seed().unwrap();
    println!("Operator public key: {}", operator_pub);

    //save operator key
    fs::write("operator.seed", operator_seed.as_bytes())?;

    //Create Account key
    let account = KeyPair::new_account();
    let account_pub = account.public_key();
    let account_seed = account.seed().unwrap();
    println!("Account public key: {}", account_pub);

    fs::write("account.seed", account_seed.as_bytes())?;

    //Create user keyu
    let user = KeyPair::new_user();
    let user_pub = user.public_key();
    let user_seed = user.seed().unwrap();

    println!("User public key: {}", user_seed);
    fs::write("user.seed", user_seed.as_bytes())?;

    println!("All keys generated successfully");

    Ok(())
}
