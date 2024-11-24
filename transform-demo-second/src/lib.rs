
// Implement the trait for each transformation module
pub mod transform_second {
    // use super::Transformer;
    use depot_common::Transformer;

    pub struct TransformDemoModuleSecond;

    impl Transformer for TransformDemoModuleSecond {
        fn transform(&self, message: &str) -> String {
            // Your transformation logic here
            message.to_lowercase() // Example transformation
        }
    }
}
