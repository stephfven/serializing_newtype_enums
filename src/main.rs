use std::path::PathBuf;

use crate::newtype_variant_enum::{
    DeviceAttributes,
    types::ControlType,
    xml::{from_xml_file, to_xml_file},
};

#[allow(dead_code)]
mod newtype_variant_enum;

fn main() {
    let file_path = PathBuf::from("test.xml");
    let obj = DeviceAttributes {
        name: "OtherDeviceTest".to_string(),
        ctrl_type: ControlType::Voltage(2.0),
    };

    println!("\n\n[Original] {}, {:?}", obj.name, obj.ctrl_type);

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
