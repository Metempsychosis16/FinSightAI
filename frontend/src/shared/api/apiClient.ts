import {
  clearAccessToken,
  getAccessToken,
} from '../../features/auth/services/tokenStorage'

function getResponseError(data: unknown): string | null {
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

export async function apiFetch<T>(
  path: string,
  options: RequestInit = {},
): Promise<T> {
  if (!path.startsWith('/') || path.startsWith('//')) {
    throw new Error('API path must start with a single slash.')
  }

  const token = getAccessToken()
  const headers = new Headers(options.headers)

  headers.set('Accept', 'application/json')

  if (options.body !== undefined && !headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json')
  }

  if (token) {
    headers.set('Authorization', `Bearer ${token}`)
  }

  let response: Response

  try {
    response = await fetch(`/api${path}`, {
      ...options,
      headers,
    })
  } catch {
    throw new Error('Could not connect to the backend.')
  }

  if (response.status === 401) {
    clearAccessToken()
    throw new Error('Your login session has expired. Please sign in again.')
  }

  if (response.status === 204) {
    return undefined as T
  }

  const data: unknown = await response.json().catch(() => null)

  if (!response.ok) {
    const message = getResponseError(data)

    throw new Error(
      message ?? `API request failed with HTTP ${response.status}.`,
    )
  }

  return data as T
}
