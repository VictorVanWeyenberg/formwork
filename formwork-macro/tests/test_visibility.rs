use formwork_macro::formwork;

mod inner {
    use super::*;

    #[formwork]
    pub struct SecretSettings {
        #[slot]
        pub(crate) nested: String,
        #[slot]
        private_flag: bool,
    }

    impl SecretSettings {
        pub fn new() -> Self {
            Self {
                nested: "test".to_string(),
                private_flag: true,
            }
        }
        pub fn get_flag(&self) -> bool {
            self.private_flag
        }
    }
}

#[test]
fn test_restricted_visibility() {
    let settings = inner::SecretSettings::new();
    assert!(settings.get_flag());
    let nested = &settings.nested;
    assert_eq!(nested, "test");
}
