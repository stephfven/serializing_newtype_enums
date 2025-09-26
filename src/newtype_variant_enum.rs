use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeviceAttributes {
    pub name: String,
    #[serde(flatten)]
    pub ctrl_type: types::ControlType,
}

pub mod types {
    use super::*;

    #[derive(Debug, PartialEq, Serialize)]
    #[serde(rename_all = "PascalCase")]
    pub enum ControlType {
        Voltage(f32),
        Power(f32),
    }

    impl<'de> Deserialize<'de> for ControlType {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            use serde::de::{self, MapAccess, Visitor};
            use std::fmt;

            // This matches either:
            //  - directly "3.5"
            //  - or { "$text": "3.5" }
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
                type Value = ControlType;

                fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.write_str("Power or Voltage element")
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
                            "Power" => Ok(ControlType::Power(f)),
                            "Voltage" => Ok(ControlType::Voltage(f)),
                            _ => Err(de::Error::custom(format!("unexpected key {}", key))),
                        };
                    }
                    Err(de::Error::custom("expected <Power> or <Voltage> element"))
                }
            }

            deserializer.deserialize_map(ControlVisitor)
        }
    }
}

pub mod xml {
    use std::{fs::File, io::BufReader, path::PathBuf};

    use super::DeviceAttributes;

    pub fn from_xml_file(file_path: impl Into<PathBuf>) -> Result<DeviceAttributes, String> {
        let file_path = file_path.into();
        let source: File = File::open(&file_path)
            .map_err(|e| format!("failed to open file: {:?}", e.to_string()))?;
        let reader: BufReader<File> = BufReader::new(source);

        let output: DeviceAttributes = quick_xml::de::from_reader(reader)
            .map_err(|e| format!("failed to deserialize: {:?}", e.to_string()))?;

        Ok(output)
    }

    pub fn to_xml_file(
        file_path: impl Into<PathBuf>,
        obj: &DeviceAttributes,
    ) -> Result<File, String> {
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
    use std::path::PathBuf;

    use super::{
        DeviceAttributes,
        types::ControlType,
        xml::{from_xml_file, to_xml_file},
    };

    #[test]
    fn export() {
        let file_path = PathBuf::from("export.xml");
        let out = DeviceAttributes {
            name: "MyDevice".to_string(),
            ctrl_type: ControlType::Power(3.5),
        };

        // <DeviceTag>
        //   <Name>MyDevice</Name>
        //   <Power>3.5</Power>
        // </DeviceTag>
        to_xml_file(&file_path, &out).expect("should have written object to file");
    }

    #[test]
    fn import() {
        let file_path = PathBuf::from("import.xml");
        let out = DeviceAttributes {
            name: "MyDevice".to_string(),
            ctrl_type: ControlType::Power(3.5),
        };

        // Must have an XML file at import.xml exactly like:
        // <DeviceTag>
        //     <Name>MyDevice</Name>
        //     <Power>3.5</Power>
        // </DeviceTag>
        let res = from_xml_file(&file_path).expect("should have read object into memory");
        assert_eq!(res, out, "imported object does not match original");
    }

    #[test]
    fn export_and_import() {
        let file_path = PathBuf::from("test.xml");
        let out = DeviceAttributes {
            name: "OtherDeviceTest".to_string(),
            ctrl_type: ControlType::Voltage(2.0),
        };

        // Export
        to_xml_file(&file_path, &out).expect("should have written object to file");

        // Import
        let res = from_xml_file(&file_path).expect("should have read object into memory");
        assert_eq!(res, out, "imported object does not match original");
    }
}
