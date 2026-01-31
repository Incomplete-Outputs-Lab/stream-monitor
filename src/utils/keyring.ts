import { getPassword, setPassword, deletePassword } from 'tauri-plugin-keyring-api';

const SERVICE_NAME = 'stream-monitor';

export async function saveToken(platform: string, token: string): Promise<void> {
  const key = `${platform}_token`;
  await setPassword(SERVICE_NAME, key, token);
  console.log(`[Keyring] Token saved for platform: ${platform}`);
}

export async function getToken(platform: string): Promise<string | null> {
  try {
    const key = `${platform}_token`;
    const token = await getPassword(SERVICE_NAME, key);
    return token;
  } catch (error) {
    console.error(`[Keyring] Failed to get token for ${platform}:`, error);
    return null;
  }
}

export async function deleteToken(platform: string): Promise<void> {
  const key = `${platform}_token`;
  await deletePassword(SERVICE_NAME, key);
  console.log(`[Keyring] Token deleted for platform: ${platform}`);
}

export async function hasToken(platform: string): Promise<boolean> {
  try {
    const token = await getToken(platform);
    return token !== null && token.length > 0;
  } catch {
    return false;
  }
}

export async function saveOAuthSecret(platform: string, secret: string): Promise<void> {
  const key = `${platform}_oauth_secret`;
  await setPassword(SERVICE_NAME, key, secret);
  console.log(`[Keyring] OAuth secret saved for platform: ${platform}`);
}

export async function getOAuthSecret(platform: string): Promise<string | null> {
  try {
    const key = `${platform}_oauth_secret`;
    const secret = await getPassword(SERVICE_NAME, key);
    return secret;
  } catch (error) {
    console.error(`[Keyring] Failed to get OAuth secret for ${platform}:`, error);
    return null;
  }
}
