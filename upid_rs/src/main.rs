use std::env;

use upid::Upid;

fn main() {
    let args: Vec<String> = env::args().collect();
    let prefix = match args.get(1) {
        Some(value) => value,
        None => &"".to_string(),
    };
    println!("{}", Upid::from_prefix(prefix).unwrap().to_string());
}
