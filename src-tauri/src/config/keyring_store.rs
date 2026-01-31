use tauri::{AppHandle, Runtime};

pub struct KeyringStore;

impl KeyringStore {
    const SERVICE_NAME: &'static str = "stream-monitor";

    /// Save a token to OS keychain
    pub fn save_token_with_app<R: Runtime>(
        app: &AppHandle<R>,
        platform: &str,
        token: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use tauri_plugin_keyring::KeyringExt;

        let key = format!("{}_token", platform);
        app.keyring()
            .set_password(Self::SERVICE_NAME, &key, token)?;

        eprintln!("[KeyringStore] Token saved for platform: '{}'", platform);
        Ok(())
    }

    /// Get a token from OS keychain
    pub fn get_token_with_app<R: Runtime>(
        app: &AppHandle<R>,
        platform: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        use tauri_plugin_keyring::KeyringExt;

        let key = format!("{}_token", platform);
        let token = app
            .keyring()
            .get_password(Self::SERVICE_NAME, &key)?
            .ok_or("Token not found")?;

        Ok(token)
    }

    /// Delete a token from OS keychain
    pub fn delete_token_with_app<R: Runtime>(
        app: &AppHandle<R>,
        platform: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use tauri_plugin_keyring::KeyringExt;

        let key = format!("{}_token", platform);
        app.keyring().delete_password(Self::SERVICE_NAME, &key)?;

        eprintln!("[KeyringStore] Token deleted for platform: '{}'", platform);
        Ok(())
    }

    /// Check if a token exists
    pub fn has_token_with_app<R: Runtime>(app: &AppHandle<R>, platform: &str) -> bool {
        Self::get_token_with_app(app, platform).is_ok()
    }

    /// Save OAuth secret
    pub fn save_oauth_secret_with_app<R: Runtime>(
        app: &AppHandle<R>,
        platform: &str,
        secret: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use tauri_plugin_keyring::KeyringExt;

        let key = format!("{}_oauth_secret", platform);
        app.keyring()
            .set_password(Self::SERVICE_NAME, &key, secret)?;

        eprintln!(
            "[KeyringStore] OAuth secret saved for platform: '{}'",
            platform
        );
        Ok(())
    }

    /// Get OAuth secret
    pub fn get_oauth_secret_with_app<R: Runtime>(
        app: &AppHandle<R>,
        platform: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        use tauri_plugin_keyring::KeyringExt;

        let key = format!("{}_oauth_secret", platform);
        let secret = app
            .keyring()
            .get_password(Self::SERVICE_NAME, &key)?
            .ok_or("Secret not found")?;

        Ok(secret)
    }
}
