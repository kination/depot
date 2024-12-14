pub mod transform {
    use std::collections::HashMap;
    use regex::Regex;
    use depot_common::Transformer;

    pub struct TransformRegexModule;

    impl Transformer for TransformRegexModule {
        fn transform(&self, message: &str, option: &Option<HashMap<String, String>>) -> Result<String, String> {
            if let Some(ref opts) = option {
                let expression = opts.get("exp").ok_or("Error: 'exp' key not found!")?;
                let regex = Regex::new(expression).unwrap();

                if let Some(caps) = regex.captures(message) {
                    let mut json_data = serde_json::json!({});

                    if let Some(ref opts) = option {
                        // Iterate over the keys in the options to build the JSON object
                        for (key, capture_name) in opts.iter() {
                            if let Some(capture) = caps.name(capture_name) {
                                json_data[key] = serde_json::json!(capture.as_str());
                            } else {
                                json_data[key] = serde_json::json!("");
                            }
                        }
                    }
                    return Ok(json_data.to_string())
                } else {
                    // println!("No matches found.");
                    // assert!(false, "Expected captures but found none.");
                    // return Err("No matches found.".to_string());
                }
            }

            Err(message.to_uppercase().to_string())
        }
    }
}

