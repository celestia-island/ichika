use syn::{parse_str, TypePath};

fn main() {
    let usize1: TypePath = parse_str("usize").unwrap();
    let usize2: TypePath = parse_str("usize").unwrap();
    let string: TypePath = parse_str("String").unwrap();
    
    println!("usize == usize: {:?}", format!("{:?}", usize1) == format!("{:?}", usize2));
    println!("usize == String: {:?}", format!("{:?}", usize1) == format!("{:?}", string));
    println!("usize1: {:?}", usize1);
    println!("usize2: {:?}", usize2);
    println!("string: {:?}", string);
}
