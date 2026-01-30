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
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(e) => {
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

    pub fn save_oauth_secret(platform: &str, secret: &str) -> Result<(), Box<dyn Error>> {
        let entry = Entry::new(SERVICE_NAME, &format!("{}_oauth_secret", platform))?;
        entry.set_password(secret)?;
        Ok(())
    }

    pub fn get_oauth_secret(platform: &str) -> Result<String, Box<dyn Error>> {
        let entry = Entry::new(SERVICE_NAME, &format!("{}_oauth_secret", platform))?;
        let secret = entry.get_password()?;
        Ok(secret)
    }

    pub fn delete_oauth_secret(platform: &str) -> Result<(), Box<dyn Error>> {
        let entry = Entry::new(SERVICE_NAME, &format!("{}_oauth_secret", platform))?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(e) => {
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

    pub fn has_oauth_secret(platform: &str) -> bool {
        Self::get_oauth_secret(platform).is_ok()
    }
}
