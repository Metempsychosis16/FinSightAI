import { useState, type FormEvent } from 'react'
import { login } from '../services/authService'
import { saveAccessToken } from '../services/tokenStorage'
import './LoginPage.css'


interface LoginPageProps {
  onAuthenticated: () => void
}

function LoginPage({ onAuthenticated }: LoginPageProps) {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [message, setMessage] = useState('')
  const [isSubmitting, setIsSubmitting] = useState(false)

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()

    setMessage('')
    setIsSubmitting(true)

    try {
      const result = await login({ email, password })

      saveAccessToken(result.access_token, result.expires_in)
	onAuthenticated()

      setPassword('')
      setMessage(
        `Sign-in successful. Your session expires in ${Math.floor(result.expires_in / 60)} minutes.`,
      )
    } catch (error: unknown) {
      const errorMessage =
        error instanceof Error ? error.message : 'An unexpected error occurred.'

      setMessage(errorMessage)
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <main className="login-page">
      <section className="login-panel" aria-labelledby="login-title">
        <div className="login-brand">
          <div className="brand-mark" aria-hidden="true">
            FS
          </div>
          <span>FinSight AI</span>
        </div>

        <div className="login-heading">
          <p className="eyebrow">INVESTIGATION PLATFORM</p>
          <h1 id="login-title">Welcome back</h1>
          <p className="login-description">
            Sign in to access your financial crime investigations.
          </p>
        </div>

        <form
          className="login-form"
          onSubmit={handleSubmit}
          aria-busy={isSubmitting}
        >
          <div className="form-field">
            <label htmlFor="email">Email address</label>
            <input
              id="email"
              name="email"
              type="email"
              autoComplete="username"
              placeholder="you@example.com"
              value={email}
              onChange={(event) => {
                setEmail(event.target.value)
                setMessage('')
              }}
              required
              disabled={isSubmitting}
            />
          </div>

          <div className="form-field">
            <label htmlFor="password">Password</label>
            <input
              id="password"
              name="password"
              type="password"
              autoComplete="current-password"
              placeholder="Enter your password"
              value={password}
              onChange={(event) => {
                setPassword(event.target.value)
                setMessage('')
              }}
              required
              disabled={isSubmitting}
            />
          </div>

          <button
            className="login-button"
            type="submit"
            disabled={isSubmitting}
          >
            {isSubmitting ? 'Signing in...' : 'Sign in'}
          </button>

          <p className="login-message" role="status" aria-live="polite">
            {message}
          </p>
        </form>

        <p className="login-footer">
          Secure access for authorized investigators
        </p>
      </section>
    </main>
  )
}

export default LoginPage
