import { useState } from 'react'
import { Lock, ShieldCheck, Eye, EyeOff } from 'lucide-react'
import { api } from '@/lib/api'
import { authStore } from '@/lib/auth'

interface Props {
  onDone: () => void
}

export function SetupPage({ onDone }: Props) {
  const [password, setPassword] = useState('')
  const [confirm, setConfirm] = useState('')
  const [showPw, setShowPw] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setError(null)

    if (password.length < 8) {
      setError('Password must be at least 8 characters.')
      return
    }
    if (password !== confirm) {
      setError('Passwords do not match.')
      return
    }

    setLoading(true)
    try {
      const res = await api.authSetup(password)
      if (!res.ok) {
        const body = await res.json().catch(() => ({}))
        setError((body as { error?: string }).error ?? 'Setup failed.')
        return
      }
      // Auto-login after setup
      const loginRes = await api.authLogin(password)
      if (loginRes.ok) {
        const { token } = await loginRes.json() as { token: string }
        authStore.setToken(token)
        onDone()
      } else {
        setError('Setup succeeded but login failed. Please refresh.')
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
          <ShieldCheck className="auth-icon-svg" />
        </div>

        <h1 className="auth-title">Welcome to Mori</h1>
        <p className="auth-subtitle">
          Set up a password to protect your dashboard. You can only do this once.
        </p>

        <form onSubmit={handleSubmit} className="auth-form">
          <div className="auth-field">
            <label htmlFor="setup-password" className="auth-label">Password</label>
            <div className="auth-input-wrap">
              <Lock className="auth-input-icon" />
              <input
                id="setup-password"
                type={showPw ? 'text' : 'password'}
                className="auth-input"
                placeholder="Choose a strong password"
                value={password}
                onChange={e => setPassword(e.target.value)}
                autoComplete="new-password"
                required
              />
              <button type="button" className="auth-eye" onClick={() => setShowPw(v => !v)} tabIndex={-1}>
                {showPw ? <EyeOff size={14} /> : <Eye size={14} />}
              </button>
            </div>
          </div>

          <div className="auth-field">
            <label htmlFor="setup-confirm" className="auth-label">Confirm Password</label>
            <div className="auth-input-wrap">
              <Lock className="auth-input-icon" />
              <input
                id="setup-confirm"
                type={showPw ? 'text' : 'password'}
                className="auth-input"
                placeholder="Repeat your password"
                value={confirm}
                onChange={e => setConfirm(e.target.value)}
                autoComplete="new-password"
                required
              />
            </div>
          </div>

          {error && <p className="auth-error">{error}</p>}

          <button type="submit" className="auth-btn" disabled={loading}>
            {loading ? 'Setting up…' : 'Create Account'}
          </button>
        </form>
      </div>
    </div>
  )
}
