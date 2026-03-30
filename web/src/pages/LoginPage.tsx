import { useState } from 'react'
import { Lock, LogIn, Eye, EyeOff } from 'lucide-react'
import { api } from '@/lib/api'
import { authStore } from '@/lib/auth'

interface Props {
  onDone: () => void
}

export function LoginPage({ onDone }: Props) {
  const [password, setPassword] = useState('')
  const [showPw, setShowPw] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setError(null)
    setLoading(true)
    try {
      const res = await api.authLogin(password)
      if (res.ok) {
        const { token } = await res.json() as { token: string }
        authStore.setToken(token)
        onDone()
      } else {
        setError('Incorrect password. Please try again.')
      }
    } catch (err) {
      setError((err as Error).message)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="auth-page">
      <div className="auth-card">
        <div className="auth-icon">
          <LogIn className="auth-icon-svg" />
        </div>

        <h1 className="auth-title">Mori<span className="text-primary">.</span></h1>
        <p className="auth-subtitle">Enter your password to continue.</p>

        <form onSubmit={handleSubmit} className="auth-form">
          <div className="auth-field">
            <label htmlFor="login-password" className="auth-label">Password</label>
            <div className="auth-input-wrap">
              <Lock className="auth-input-icon" />
              <input
                id="login-password"
                type={showPw ? 'text' : 'password'}
                className="auth-input"
                placeholder="Your password"
                value={password}
                onChange={e => setPassword(e.target.value)}
                autoComplete="current-password"
                autoFocus
                required
              />
              <button type="button" className="auth-eye" onClick={() => setShowPw(v => !v)} tabIndex={-1}>
                {showPw ? <EyeOff size={14} /> : <Eye size={14} />}
              </button>
            </div>
          </div>

          {error && <p className="auth-error">{error}</p>}

          <button type="submit" className="auth-btn" disabled={loading}>
            {loading ? 'Signing in…' : 'Sign In'}
          </button>
        </form>
      </div>
    </div>
  )
}
