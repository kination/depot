// use regex::Regex;

// Define a trait for transformation
// pub trait Transformer {
//     fn transform(&self, message: &str) -> String;
// }

// Implement the trait for each transformation module
pub mod transform {
    // use super::Transformer;
    use depot_common::Transformer;

    pub struct TransformDemoModule;

    impl Transformer for TransformDemoModule {
        fn transform(&self, message: &str) -> String {
            // Your transformation logic here
            message.to_uppercase() // Example transformation
        }
    }
}

