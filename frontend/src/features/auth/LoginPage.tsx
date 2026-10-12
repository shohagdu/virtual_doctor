import { useActionState } from 'react'
import { Navigate, useNavigate } from 'react-router'
import { useTranslation } from 'react-i18next'
import { api } from '../../api/client'
import type { LoginResponse } from '../../api/types'
import { useAuth } from './authStore'

export function LoginPage() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const { token, setSession } = useAuth()

  const [error, submit, pending] = useActionState(async (_prev: string | null, form: FormData) => {
    try {
      const res = await api<LoginResponse>('/auth/login', {
        method: 'POST',
        body: JSON.stringify({ email: form.get('email'), password: form.get('password') }),
      })
      setSession(res.access_token, res.user)
      navigate('/dashboard')
      return null
    } catch {
      return t('auth.failed')
    }
  }, null)

  if (token) return <Navigate to="/dashboard" replace />

  return (
    <div className="flex min-h-screen items-center justify-center p-4">
      <form action={submit} className="w-full max-w-sm space-y-4 rounded-lg bg-white p-6 shadow">
        <h1 className="text-xl font-semibold text-brand">{t('app.name')}</h1>
        <label className="block text-sm">
          {t('auth.email')}
          <input name="email" type="email" required className="mt-1 w-full rounded border px-3 py-2" />
        </label>
        <label className="block text-sm">
          {t('auth.password')}
          <input name="password" type="password" required className="mt-1 w-full rounded border px-3 py-2" />
        </label>
        {error && <p className="text-sm text-red-600">{error}</p>}
        <button disabled={pending} className="w-full rounded bg-brand py-2 text-white disabled:opacity-60">
          {t('auth.login')}
        </button>
      </form>
    </div>
  )
}
