use playfair::PlayfairCipher;

fn main() {
    let key = "my own little secret";
    let cipher = PlayfairCipher::new(key);
    let encrypted_message = "Rzie tt debtnwl. Dwm'e veseqt cmowmb!w";
    let secret_message = cipher.decode(encrypted_message).unwrap();
    println!("key: {}", key);
    println!("encrypted message: {}", encrypted_message);
    println!(
        "secret message: {}",
        secret_message
            .chars()
            .filter(|&c| c != 'x')
            .collect::<String>()
    );
}
