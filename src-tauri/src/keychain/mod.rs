#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(not(target_os = "macos"))]
pub mod macos {
    pub struct KeychainManager {
        _service: String,
    }

    impl KeychainManager {
        pub fn new(service: &str) -> Self {
            Self {
                _service: service.to_string(),
            }
        }

        pub fn store(
            &self,
            _account: &str,
            _password: &str,
        ) -> Result<(), Box<dyn std::error::Error>> {
            Err("Keychain is only available on macOS".into())
        }

        pub fn retrieve(
            &self,
            _account: &str,
        ) -> Result<Option<String>, Box<dyn std::error::Error>> {
            Err("Keychain is only available on macOS".into())
        }

        pub fn delete(&self, _account: &str) -> Result<(), Box<dyn std::error::Error>> {
            Err("Keychain is only available on macOS".into())
        }
    }
}
