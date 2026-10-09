use schematic_macro::schematic;

mod inner {
    use super::*;

    #[schematic]
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
    }
}

fn main() {
    let settings = inner::SecretSettings::new();
    let flag = settings.private_flag; // Should fail
}
