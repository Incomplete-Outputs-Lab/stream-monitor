import { Stronghold, Client } from '@tauri-apps/plugin-stronghold';
import { appDataDir } from '@tauri-apps/api/path';
import { listen } from '@tauri-apps/api/event';

const CLIENT_NAME = 'stream-stats-collector';

let strongholdInstance: Stronghold | null = null;
let clientInstance: Client | null = null;

// Listen for backend events to save/load tokens
export function setupStrongholdEventListeners() {
  // Save token event from Rust
  listen<{ platform: string; token: string }>('stronghold:save-token', async (event) => {
    const { platform, token } = event.payload;
    console.log(`[Stronghold] Received save-token event for platform: ${platform}`);
    
    if (clientInstance && strongholdInstance) {
      try {
        await saveToken(platform, token);
        console.log(`[Stronghold] Token saved for ${platform}`);
      } catch (error) {
        console.error(`[Stronghold] Failed to save token for ${platform}:`, error);
      }
    } else {
      console.error('[Stronghold] Cannot save token - vault not unlocked');
    }
  });

  // Delete token event
  listen<{ platform: string }>('stronghold:delete-token', async (event) => {
    const { platform } = event.payload;
    console.log(`[Stronghold] Received delete-token event for platform: ${platform}`);
    
    if (clientInstance && strongholdInstance) {
      try {
        await deleteToken(platform);
        console.log(`[Stronghold] Token deleted for ${platform}`);
      } catch (error) {
        console.error(`[Stronghold] Failed to delete token for ${platform}:`, error);
      }
    }
  });

  // Save secret event
  listen<{ platform: string; secret: string }>('stronghold:save-secret', async (event) => {
    const { platform, secret } = event.payload;
    console.log(`[Stronghold] Received save-secret event for platform: ${platform}`);
    
    if (clientInstance && strongholdInstance) {
      try {
        await saveOAuthSecret(platform, secret);
        console.log(`[Stronghold] Secret saved for ${platform}`);
      } catch (error) {
        console.error(`[Stronghold] Failed to save secret for ${platform}:`, error);
      }
    }
  });

  // Delete secret event
  listen<{ platform: string }>('stronghold:delete-secret', async (event) => {
    const { platform } = event.payload;
    console.log(`[Stronghold] Received delete-secret event for platform: ${platform}`);
    
    if (clientInstance && strongholdInstance) {
      try {
        const key = `${platform}_oauth_secret`;
        const store = clientInstance.getStore();
        await store.remove(key);
        await strongholdInstance.save();
        console.log(`[Stronghold] Secret deleted for ${platform}`);
      } catch (error) {
        console.error(`[Stronghold] Failed to delete secret for ${platform}:`, error);
      }
    }
  });

  console.log('[Stronghold] Event listeners setup complete');
}

/**
 * Initialize or load the Stronghold vault
 */
export async function initializeStronghold(password: string): Promise<void> {
  const vaultPath = `${await appDataDir()}vault.hold`;
  
  console.log('[Stronghold] Initializing vault at:', vaultPath);
  
  strongholdInstance = await Stronghold.load(vaultPath, password);
  
  // Try to load existing client or create new one
  try {
    clientInstance = await strongholdInstance.loadClient(CLIENT_NAME);
    console.log('[Stronghold] Loaded existing client');
  } catch {
    clientInstance = await strongholdInstance.createClient(CLIENT_NAME);
    console.log('[Stronghold] Created new client');
  }
}

/**
 * Check if vault is initialized
 */
export async function isVaultInitialized(): Promise<boolean> {
  try {
    // Use Tauri command to check vault initialization
    const { invoke } = await import('@tauri-apps/api/core');
    const status = await invoke<{ initialized: boolean }>('check_vault_initialized');
    return status.initialized;
  } catch (error) {
    console.error('[Stronghold] Failed to check vault initialization:', error);
    return false;
  }
}

/**
 * Save a token to the vault
 */
export async function saveToken(platform: string, token: string): Promise<void> {
  if (!clientInstance || !strongholdInstance) {
    throw new Error('Stronghold not initialized. Please unlock the vault first.');
  }
  
  const key = `${platform}_token`;
  const store = clientInstance.getStore();
  const data = Array.from(new TextEncoder().encode(token));
  
  await store.insert(key, data);
  await strongholdInstance.save();
  
  console.log(`[Stronghold] Token saved for platform: ${platform}`);
}

/**
 * Get a token from the vault
 */
export async function getToken(platform: string): Promise<string | null> {
  if (!clientInstance) {
    throw new Error('Stronghold not initialized. Please unlock the vault first.');
  }
  
  const key = `${platform}_token`;
  const store = clientInstance.getStore();
  
  try {
    const data = await store.get(key);
    if (data) {
      return new TextDecoder().decode(new Uint8Array(data));
    }
  } catch (error) {
    console.error(`[Stronghold] Failed to get token for ${platform}:`, error);
  }
  
  return null;
}

/**
 * Delete a token from the vault
 */
export async function deleteToken(platform: string): Promise<void> {
  if (!clientInstance || !strongholdInstance) {
    throw new Error('Stronghold not initialized. Please unlock the vault first.');
  }
  
  const key = `${platform}_token`;
  const store = clientInstance.getStore();
  
  await store.remove(key);
  await strongholdInstance.save();
  
  console.log(`[Stronghold] Token deleted for platform: ${platform}`);
}

/**
 * Check if a token exists in the vault
 */
export async function hasToken(platform: string): Promise<boolean> {
  if (!clientInstance) {
    return false;
  }
  
  const key = `${platform}_token`;
  const store = clientInstance.getStore();
  
  try {
    const data = await store.get(key);
    return data !== null && data !== undefined && data.length > 0;
  } catch (error) {
    console.error(`[Stronghold] Failed to check token for ${platform}:`, error);
    return false;
  }
}

/**
 * Check if vault is currently unlocked
 */
export function isVaultUnlocked(): boolean {
  return strongholdInstance !== null && clientInstance !== null;
}

/**
 * Lock the vault
 */
export function lockVault(): void {
  strongholdInstance = null;
  clientInstance = null;
  console.log('[Stronghold] Vault locked');
}

/**
 * Save OAuth secret
 */
export async function saveOAuthSecret(platform: string, secret: string): Promise<void> {
  if (!clientInstance || !strongholdInstance) {
    throw new Error('Stronghold not initialized. Please unlock the vault first.');
  }
  
  const key = `${platform}_oauth_secret`;
  const store = clientInstance.getStore();
  const data = Array.from(new TextEncoder().encode(secret));
  
  await store.insert(key, data);
  await strongholdInstance.save();
  
  console.log(`[Stronghold] OAuth secret saved for platform: ${platform}`);
}

/**
 * Get OAuth secret
 */
export async function getOAuthSecret(platform: string): Promise<string | null> {
  if (!clientInstance) {
    throw new Error('Stronghold not initialized. Please unlock the vault first.');
  }
  
  const key = `${platform}_oauth_secret`;
  const store = clientInstance.getStore();
  
  try {
    const data = await store.get(key);
    if (data) {
      return new TextDecoder().decode(new Uint8Array(data));
    }
  } catch (error) {
    console.error(`[Stronghold] Failed to get OAuth secret for ${platform}:`, error);
  }
  
  return null;
}
