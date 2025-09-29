use serde::{Deserialize, Serialize};

use crate::newtype_variant_enum::types::{
    parse_sale_or_empty_string, serialize_currency::deserialize_flattened,
};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Product {
    pub name: String,
    #[serde(flatten, deserialize_with = "deserialize_flattened")]
    pub price: types::Currency,
    #[serde(deserialize_with = "parse_sale_or_empty_string")]
    pub sale: Option<types::Sale>,
}

pub mod types {
    use serde::de::{Deserializer, Error};

    use super::*;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct Sale(pub f32);

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub enum Currency {
        Dollars(f32),
        Euros(f32),
    }
    pub mod serialize_currency {
        use super::*;
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum TextOrMap {
            Text(String),
            Map {
                #[serde(rename = "$text")]
                text: String,
            },
        }

        struct ControlVisitor;

        impl<'de> Visitor<'de> for ControlVisitor {
            type Value = Currency;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("Euros or Dollars element")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                while let Some((key, tom)) = map.next_entry::<String, TextOrMap>()? {
                    // Extract the string content
                    let s = match tom {
                        TextOrMap::Text(t) => t,
                        TextOrMap::Map { text } => text,
                    };
                    let f = s.parse::<f32>().map_err(de::Error::custom)?;
                    return match key.as_str() {
                        "Euros" => Ok(Currency::Euros(f)),
                        "Dollars" => Ok(Currency::Dollars(f)),
                        _ => Err(de::Error::custom(format!("unexpected key {}", key))),
                    };
                }
                Err(de::Error::custom("expected <Euros> or <Dollars> element"))
            }
        }

        pub fn deserialize_flattened<'de, D>(deserializer: D) -> Result<Currency, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            deserializer.deserialize_map(ControlVisitor)
        }
    }

    pub fn parse_sale_or_empty_string<'de, D>(deserializer: D) -> Result<Option<Sale>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // WORKING but not elegant - try this approach using an enum: https://users.rust-lang.org/t/serde-deserialize-empty-string-as-option-none/116201/2
        match String::deserialize(deserializer) {
            Ok(str) if str.is_empty() => Ok(None),
            Ok(str) => Ok(Some(Sale(str.parse::<f32>().map_err(|err| {
                D::Error::custom(format!("unexpected nonempty string: `{err}`"))
            })?))),
            Err(err) => Err(err),
        }
    }
}

pub mod xml {
    use std::{fs::File, io::BufReader, path::PathBuf};

    use super::Product;

    pub fn from_xml_file(file_path: impl Into<PathBuf>) -> Result<Product, String> {
        let file_path = file_path.into();
        let source: File = File::open(&file_path)
            .map_err(|e| format!("failed to open file: {:?}", e.to_string()))?;
        let reader: BufReader<File> = BufReader::new(source);

        let output: Product = quick_xml::de::from_reader(reader)
            .map_err(|e| format!("failed to deserialize: {:?}", e.to_string()))?;

        Ok(output)
    }

    pub fn to_xml_file(file_path: impl Into<PathBuf>, obj: &Product) -> Result<File, String> {
        let file: File = File::create(file_path.into())
            .map_err(|e| format!("failed to create file: {:?}", e.to_string()))?;
        let mut writer: quick_xml::Writer<&File> = quick_xml::Writer::new(&file);

        writer
            .write_serializable("DeviceTag", obj)
            .map_err(|e| format!("failed to serialize: {:?}", e.to_string()))?;

        Ok(file)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::newtype_variant_enum::types::Sale;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    use super::{
        Product,
        types::Currency,
        xml::{from_xml_file, to_xml_file},
    };

    #[test]
    fn export_only() {
        let file_path = PathBuf::from("export.xml");
        let out = Product {
            name: "Fidget Spinner".to_string(),
            price: Currency::Euros(3.5),
            sale: None,
        };

        to_xml_file(&file_path, &out).expect("should have written object to file");
    }

    #[test]
    fn import_only() {
        let file_path = PathBuf::from("import.xml");
        let exp = Product {
            name: "Fidget Spinner".to_string(),
            price: Currency::Euros(3.5),
            sale: None,
        };

        let res = from_xml_file(&file_path).expect("should have read object into memory");
        assert_eq!(res, exp, "imported object does not match original");
    }

    #[test]
    fn both_export_and_import_without_rating() {
        let file_path = PathBuf::from("test.xml");
        let obj = Product {
            name: "F-22 Raptor".to_string(),
            price: Currency::Dollars(350000000.0),
            sale: None,
        };

        // Export
        to_xml_file(&file_path, &obj).expect("should have written object to file");

        // Import
        let res = from_xml_file(&file_path).expect("should have read object into memory");
        assert_eq!(res, obj, "imported object does not match original");
    }

    #[test]
    fn both_export_and_import_with_rating() {
        let file_path = PathBuf::from("test1.xml");
        let obj = Product {
            name: "Scrub Daddy".to_string(),
            price: Currency::Dollars(6.0),
            sale: Some(Sale(25.5)),
        };

        // Export
        to_xml_file(&file_path, &obj).expect("should have written object to file");

        // Import
        let res = from_xml_file(&file_path).expect("should have read object into memory");
        assert_eq!(res, obj, "imported object does not match original");
    }
}
