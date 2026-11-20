import type { LoginRequest, LoginResponse } from '../../../shared/types/auth'

function isLoginResponse(data: unknown): data is LoginResponse {
  if (typeof data !== 'object' || data === null) {
    return false
  }

  const response = data as Record<string, unknown>

  return (
    typeof response.access_token === 'string' &&
    response.access_token.length > 0 &&
    response.token_type === 'Bearer' &&
    typeof response.expires_in === 'number' &&
    Number.isFinite(response.expires_in) &&
    response.expires_in > 0
  )
}

function getErrorMessage(data: unknown): string | null {
  if (typeof data !== 'object' || data === null) {
    return null
  }

  const body = data as Record<string, unknown>

  if (typeof body.message === 'string') {
    return body.message
  }

  if (typeof body.error === 'string') {
    return body.error
  }

  return null
}

export async function login(
  credentials: LoginRequest,
): Promise<LoginResponse> {
  let response: Response

  try {
    response = await fetch('/api/auth/login', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Accept: 'application/json',
      },
      body: JSON.stringify(credentials),
    })
  } catch {
    throw new Error(
      'Could not connect to the backend. Check that the Rust server is running.',
    )
  }

  const data: unknown = await response.json().catch(() => null)

  if (!response.ok) {
    const message = getErrorMessage(data)

    throw new Error(
      message ?? `Login failed. The server returned HTTP ${response.status}.`,
    )
  }

  if (!isLoginResponse(data)) {
    throw new Error('The backend returned an invalid login response.')
  }

  return data
}
