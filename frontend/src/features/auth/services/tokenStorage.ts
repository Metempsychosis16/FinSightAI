
const ACCESS_TOKEN_KEY = 'finsight_access_token'
const TOKEN_EXPIRY_KEY = 'finsight_access_token_expiry'

export function saveAccessToken(
  token: string,
  expiresInSeconds: number,
): void {
  if (
    !token ||
    !Number.isFinite(expiresInSeconds) ||
    expiresInSeconds <= 0
  ) {
    throw new Error('Invalid access token or expiry time.')
  }

  try {
    const expiresAt = Date.now() + expiresInSeconds * 1000

    sessionStorage.setItem(ACCESS_TOKEN_KEY, token)
    sessionStorage.setItem(TOKEN_EXPIRY_KEY, String(expiresAt))
  } catch {
    clearAccessToken()
    throw new Error('Unable to store the login session in this browser.')
  }
}

export function getAccessToken(): string | null {
  try {
    const token = sessionStorage.getItem(ACCESS_TOKEN_KEY)
    const expiry = sessionStorage.getItem(TOKEN_EXPIRY_KEY)

    if (!token || !expiry) {
      clearAccessToken()
      return null
    }

    const expiresAt = Number(expiry)

    if (!Number.isFinite(expiresAt) || Date.now() >= expiresAt) {
      clearAccessToken()
      return null
    }

    return token
  } catch {
    throw new Error('Unable to read the login session from this browser.')
  }
}

export function clearAccessToken(): void {
  try {
    sessionStorage.removeItem(ACCESS_TOKEN_KEY)
    sessionStorage.removeItem(TOKEN_EXPIRY_KEY)
  } catch {
    throw new Error('Unable to clear the login session from this browser.')
  }
}

