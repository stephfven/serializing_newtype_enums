use std::path::PathBuf;

use crate::newtype_variant_enum::{
    Product,
    types::{Currency, Sale},
    xml::{from_xml_file, to_xml_file},
};

#[allow(dead_code)]
mod newtype_variant_enum;

fn main() {
    let file_path = PathBuf::from("test.xml");
    let obj = Product {
        name: "Scrub Daddy".to_string(),
        price: Currency::Dollars(6.0),
        sale: Some(Sale(25.5)),
    };

    println!("\n\nWith a rating:");
    println!("\n[Original] {}, {:?}, {:?}", obj.name, obj.price, obj.sale);

    // Export
    to_xml_file(&file_path, &obj).expect("should have written object to file");
    println!("\n\n[EXPORT] Successfully wrote object to file.");

    // Import
    let res = from_xml_file(&file_path).expect("should have read object into memory");
    println!("\n\n[IMPORT] Successfully imported object from file.");

    assert_eq!(res, obj, "imported object does not match original");
    println!("\n[IMPORT] Imported object matches original.");

    println!("\n\n~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~\n\n");

    let file_path = PathBuf::from("test.xml");
    let obj = Product {
        name: "F-22 Raptor".to_string(),
        price: Currency::Dollars(350000000.0),
        sale: None,
    };

    println!("Without a rating:");
    println!("\n[Original] {}, {:?}, {:?}", obj.name, obj.price, obj.sale);

    // Export
    to_xml_file(&file_path, &obj).expect("should have written object to file");
    println!("\n\n[EXPORT] Successfully wrote object to file.");

    // Import
    let res = from_xml_file(&file_path).expect("should have read object into memory");
    println!("\n\n[IMPORT] Successfully imported object from file.");

    assert_eq!(res, obj, "imported object does not match original");
    println!("\n[IMPORT] Imported object matches original.");

    println!("\n\n");
}
