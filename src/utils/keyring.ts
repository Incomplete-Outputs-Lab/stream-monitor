import { getPassword } from 'tauri-plugin-keyring-api';

const SERVICE_NAME = 'stream-monitor';

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

export async function hasToken(platform: string): Promise<boolean> {
  try {
    const token = await getToken(platform);
    return token !== null && token.length > 0;
  } catch {
    return false;
  }
}
