// Copyright (c) 2019-2025 Provable Inc.
// This file is part of the snarkOS library.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at:

// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use snarkvm::prelude::{Network, Plaintext, Value};

use anyhow::Result;

/// Convert a `Value` to JSON.
/// `Value` implements Serialize by just calling `to_string()` on the value, which leads to Display
/// output which (sometimes) looks like JSON but isn't.
/// We produce actual JSON by calling `serde_json::to_value` on the inner object.
pub(crate) fn value_to_json<N: Network>(mapping_value: Option<&Value<N>>) -> Result<serde_json::Value> {
    let json_value = if let Some(mapping_value) = mapping_value {
        match mapping_value {
            Value::Plaintext(plaintext) => match plaintext {
                Plaintext::Array(array, _) => serde_json::to_value(array)?,
                Plaintext::Struct(map, _) => serde_json::to_value(map)?,
                Plaintext::Literal(literal, _) => serde_json::to_value(literal)?,
            },
            Value::Record(record) => serde_json::to_value(record)?,
            Value::Future(future) => serde_json::to_value(future)?,
        }
    } else {
        serde_json::Value::Object(Default::default())
    };

    Ok(json_value)
}

pub(crate) fn mapping_to_json<N: Network>(mapping_values: &[(Plaintext<N>, Value<N>)]) -> Result<serde_json::Value> {
    let mut json = serde_json::Map::new();
    for (key, value) in mapping_values {
        let key = match key {
            // TODO: not sure if keys can be anything other than literals
            Plaintext::Array(array, _) => serde_json::to_string(&array)?,
            Plaintext::Struct(map, _) => serde_json::to_string(&map)?,
            // We can not use json here as it would cause the key to be escaped
            Plaintext::Literal(literal, _) => literal.to_string(),
        };
        json.insert(key, value_to_json(Some(value))?);
    }

    Ok(serde_json::Value::Object(json))
}
