use security_framework::passwords::{
    delete_generic_password, get_generic_password, set_generic_password,
};

pub struct KeychainManager {
    service: String,
}

impl KeychainManager {
    pub fn new(service: &str) -> Self {
        Self {
            service: service.to_string(),
        }
    }

    pub fn store(&self, account: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
        set_generic_password(&self.service, account, password.as_bytes())?;
        Ok(())
    }

    pub fn retrieve(
        &self,
        account: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match get_generic_password(&self.service, account) {
            Ok(bytes) => Ok(Some(String::from_utf8(bytes)?)),
            Err(e) if e.code() == -25300 => Ok(None), // errSecItemNotFound
            Err(e) => Err(Box::new(e)),
        }
    }

    pub fn delete(&self, account: &str) -> Result<(), Box<dyn std::error::Error>> {
        match delete_generic_password(&self.service, account) {
            Ok(()) => Ok(()),
            Err(e) if e.code() == -25300 => Ok(()), // errSecItemNotFound: already gone
            Err(e) => Err(Box::new(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SERVICE: &str = "com.prompter.test";

    #[test]
    fn test_store_and_retrieve_key() {
        let keychain = KeychainManager::new(TEST_SERVICE);
        keychain.store("test_openai_key", "sk-test-12345").unwrap();
        let retrieved = keychain.retrieve("test_openai_key").unwrap();
        assert_eq!(retrieved, Some("sk-test-12345".to_string()));
        let _ = keychain.delete("test_openai_key");
    }

    #[test]
    fn test_retrieve_missing_key_returns_none() {
        let keychain = KeychainManager::new(TEST_SERVICE);
        let result = keychain.retrieve("nonexistent_key").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_delete_key() {
        let keychain = KeychainManager::new(TEST_SERVICE);
        keychain.store("test_delete_key", "value").unwrap();
        keychain.delete("test_delete_key").unwrap();
        let result = keychain.retrieve("test_delete_key").unwrap();
        assert_eq!(result, None);
    }
}
