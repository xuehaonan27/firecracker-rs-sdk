use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KernelArgs(pub HashMap<String, Option<String>>);

// serialize the kernelArgs back to a string that can be provided
// to the kernel
impl ToString for KernelArgs {
    fn to_string(&self) -> String {
        let mut fields: Vec<String> = Vec::new();
        self.0.iter().for_each(|(key, value)| {
            let mut field = key.to_owned();
            if let Some(s) = value.as_ref() {
                field.push_str("=");
                field += s;
            }
            fields.push(field);
        });
        fields.join(" ")
    }
}

// deserialize the provided string to a kernelArgs map
impl From<String> for KernelArgs {
    fn from(raw_string: String) -> Self {
        let mut arg_map: HashMap<String, Option<String>> = HashMap::new();
        raw_string.split_ascii_whitespace().for_each(|kv_pair| {
            if let Some((key, value)) = kv_pair.split_once("=") {
                arg_map.insert(key.into(), Some(value.into()));
            } else {
                arg_map.insert(kv_pair.into(), None);
            }
        }); 
        
        Self(arg_map)
    }
}