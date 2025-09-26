use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeviceAttributes {
    pub name: String,
    #[serde(flatten)]
    pub ctrl_type: types::ControlType,
}

pub mod types {
    use super::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct Voltage(#[serde(rename = "$text")] pub f32);

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct Power(#[serde(rename = "$text")] pub f32);

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub enum ControlType {
        Voltage(#[serde(rename = "$text")] Voltage),
        Power(#[serde(rename = "$text")] Power),
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
    #[allow(dead_code)]
    use std::path::PathBuf;

    use super::{
        DeviceAttributes,
        types::{ControlType, Power, Voltage},
        xml::{from_xml_file, to_xml_file},
    };

    // Currently passes
    #[test]
    fn export() {
        let file_path = PathBuf::from("export.xml");
        let out = DeviceAttributes {
            name: "MyDevice".to_string(),
            ctrl_type: ControlType::Voltage(Voltage(2.5)),
        };

        // Should export as the following:
        //      <DeviceTag>
        //          <Name>MyDevice</Name>
        //          <Voltage>2.5</Voltage>
        //      </DeviceTag>
        // Where `ControlType` is flattened to either a `<Voltage>X</Voltage>`
        // or `<Voltage>2.5</Voltage>` element, without being encapsulated by an
        // outer `<ControlType>` tag.

        to_xml_file(&file_path, &out).expect("should have written object to file");
    }

    // Currently fails with error output:
    // should have read object into memory: "failed to deserialize: \"invalid type: map, expected f32\""
    #[test]
    fn import() {
        let file_path = PathBuf::from("import.xml");
        let out = DeviceAttributes {
            name: "MyDevice".to_string(),
            ctrl_type: ControlType::Power(Power(3.2)),
        };

        // Should be able to read an XML file that looks something like:
        //      <DeviceTag>
        //          <Name>MyDevice</Name>
        //          <Power>3.2</Power>
        //      </DeviceTag>
        // Where the `<Power>` tag is deserialized as `ControlType::Power`

        let res = from_xml_file(&file_path).expect("should have read object into memory");
        assert_eq!(res, out, "imported object does not match original");
    }
}
