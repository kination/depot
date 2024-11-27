
// Implement the trait for each transformation module
pub mod transform {
    use depot_common::Transformer;
    use std::collections::HashMap;

    pub struct TransformDemoModule;

    impl Transformer for TransformDemoModule {
        fn transform(&self, message: &str, option: Option<HashMap<String, String>>) -> Result<String, String> {
            // TODO: Your transformation logic here
            Ok(message.to_lowercase())
        }
    }
}
