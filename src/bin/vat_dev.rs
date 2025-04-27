use vat::package::{Package, Vat};
use std::path::PathBuf;
fn main(){
    // let package = Package::new("test".to_string(), "0.1.0".to_string());
    // let vat = Vat::new(package);
    // vat.save(&PathBuf::from("Z:\\temp"));

    // read
    let vat = Vat::read(&PathBuf::from("Z:\\temp"));
    // let vat = vat.unwrap();
    // vat.save(&PathBuf::from("Z:\\temp"));
    println!("{:?}", vat);

}