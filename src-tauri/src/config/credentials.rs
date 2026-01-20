use keyring::Entry;
use std::error::Error;

const SERVICE_NAME: &str = "stream-stats-collector";

pub struct CredentialManager;

impl CredentialManager {
    pub fn save_token(platform: &str, token: &str) -> Result<(), Box<dyn Error>> {
        let entry = Entry::new(SERVICE_NAME, &format!("{}_token", platform))?;
        entry.set_password(token)?;
        Ok(())
    }

    pub fn get_token(platform: &str) -> Result<String, Box<dyn Error>> {
        let entry = Entry::new(SERVICE_NAME, &format!("{}_token", platform))?;
        let token = entry.get_password()?;
        Ok(token)
    }

    pub fn delete_token(platform: &str) -> Result<(), Box<dyn Error>> {
        let entry = Entry::new(SERVICE_NAME, &format!("{}_token", platform))?;
        // keyring 3.xでは delete_credential を使用
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(e) => {
                // 既に存在しない場合は正常終了
                if e.to_string().contains("No such credential")
                    || e.to_string().contains("not found")
                    || e.to_string().contains("does not exist")
                {
                    Ok(())
                } else {
                    Err(Box::new(e))
                }
            }
        }
    }

    pub fn has_token(platform: &str) -> bool {
        Self::get_token(platform).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Keyring tests are unreliable on CI/CD environments. Run manually with: cargo test -- --ignored"]
    fn test_credential_manager_roundtrip() {
        let platform = "test_platform";
        let test_token = "test_token_12345";

        // クリーンアップ: 既存のトークンを削除（あれば）
        let cleanup_result = CredentialManager::delete_token(platform);
        println!("Cleanup result: {:?}", cleanup_result);

        // トークンを保存（リトライ付き）
        let mut save_attempts = 0;
        const MAX_SAVE_ATTEMPTS: usize = 3;

        let save_success = loop {
            save_attempts += 1;
            println!("Save attempt {}/{}", save_attempts, MAX_SAVE_ATTEMPTS);

            match CredentialManager::save_token(platform, test_token) {
                Ok(()) => {
                    println!("Save successful");
                    break true;
                }
                Err(e) => {
                    println!("Save failed (attempt {}): {}", save_attempts, e);
                    if save_attempts >= MAX_SAVE_ATTEMPTS {
                        break false;
                    }
                    // Windows Credential Managerの永続化待ち
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        };

        assert!(
            save_success,
            "Failed to save token after {} attempts",
            MAX_SAVE_ATTEMPTS
        );

        // Windows環境では保存後に少し待機
        #[cfg(target_os = "windows")]
        std::thread::sleep(std::time::Duration::from_millis(200));

        // トークンを取得（リトライ付き）
        let mut get_attempts = 0;
        const MAX_GET_ATTEMPTS: usize = 5;

        let retrieved = loop {
            get_attempts += 1;
            println!("Get attempt {}/{}", get_attempts, MAX_GET_ATTEMPTS);

            match CredentialManager::get_token(platform) {
                Ok(token) => {
                    println!("Get successful: {}", token);
                    break Some(token);
                }
                Err(e) => {
                    println!("Get failed (attempt {}): {}", get_attempts, e);
                    if get_attempts >= MAX_GET_ATTEMPTS {
                        break None;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
            }
        };

        assert!(
            retrieved.is_some(),
            "Failed to retrieve token after {} attempts",
            MAX_GET_ATTEMPTS
        );
        assert_eq!(retrieved.unwrap(), test_token);

        // トークンの存在確認
        assert!(CredentialManager::has_token(platform), "Token should exist");

        // クリーンアップ
        let cleanup_result = CredentialManager::delete_token(platform);
        println!("Final cleanup result: {:?}", cleanup_result);
    }

    #[test]
    #[ignore = "Keyring tests are unreliable on CI/CD environments. Run manually with: cargo test -- --ignored"]
    fn test_credential_manager_nonexistent() {
        let platform = "nonexistent_platform";

        // 存在しないトークンを取得しようとする
        assert!(CredentialManager::get_token(platform).is_err());
        assert!(!CredentialManager::has_token(platform));
    }
}
