import { useActionState, useState } from 'react'
import { Navigate, useNavigate } from 'react-router'
import { useTranslation } from 'react-i18next'
import { BrainCircuit, CalendarCheck, Eye, EyeOff, FileText, Languages, Lock, Mail, Stethoscope } from 'lucide-react'
import { api } from '../../api/client'
import type { LoginResponse } from '../../api/types'
import { useAuth } from './authStore'

export function LoginPage() {
  const { t, i18n } = useTranslation()
  const navigate = useNavigate()
  const { token, setSession } = useAuth()
  const [show, setShow] = useState(false)

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

  const features = [
    { icon: FileText, key: 'login.f1' },
    { icon: CalendarCheck, key: 'login.f2' },
    { icon: BrainCircuit, key: 'login.f3' },
  ]

  return (
    <div className="grid min-h-screen lg:grid-cols-2">
      <section className="relative hidden overflow-hidden bg-gradient-to-br from-brand via-[#1f7fcf] to-sky-500 p-12 text-white lg:flex lg:flex-col">
        <div className="absolute -top-24 -right-24 h-80 w-80 rounded-full bg-white/10" />
        <div className="absolute -bottom-32 -left-16 h-96 w-96 rounded-full bg-white/5" />
        <div className="relative flex items-center gap-3">
          <span className="flex h-11 w-11 items-center justify-center rounded-2xl bg-white/15 ring-1 ring-white/30">
            <Stethoscope className="h-6 w-6" />
          </span>
          <span className="text-xl font-bold">{t('app.name')}</span>
        </div>
        <div className="relative mt-auto max-w-md space-y-6">
          <h1 className="text-4xl leading-tight font-bold">{t('login.headline')}</h1>
          <p className="text-lg text-white/80">{t('login.sub')}</p>
          <ul className="space-y-3">
            {features.map(({ icon: Icon, key }) => (
              <li key={key} className="flex items-center gap-3">
                <span className="flex h-9 w-9 items-center justify-center rounded-xl bg-white/15">
                  <Icon className="h-5 w-5" />
                </span>
                {t(key)}
              </li>
            ))}
          </ul>
        </div>
        <p className="relative mt-12 text-sm text-white/60">© {t('app.name')}</p>
      </section>

      <section className="flex items-center justify-center p-6">
        <div className="w-full max-w-sm">
          <div className="mb-8 flex items-center gap-3 lg:hidden">
            <span className="flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-brand to-sky-500 text-white">
              <Stethoscope className="h-5 w-5" />
            </span>
            <span className="text-xl font-bold text-slate-900">{t('app.name')}</span>
          </div>
          <h2 className="text-2xl font-bold text-slate-900">{t('login.welcome')}</h2>
          <p className="mt-1 text-sm text-slate-500">{t('login.prompt')}</p>
          <form action={submit} className="mt-8 space-y-4">
            <label className="block text-sm font-medium text-slate-700">
              {t('auth.email')}
              <span className="relative mt-1.5 block">
                <Mail className="pointer-events-none absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-slate-400" />
                <input name="email" type="email" required autoComplete="username" className="w-full rounded-lg border bg-white py-2.5 pr-3 pl-9" />
              </span>
            </label>
            <label className="block text-sm font-medium text-slate-700">
              {t('auth.password')}
              <span className="relative mt-1.5 block">
                <Lock className="pointer-events-none absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-slate-400" />
                <input
                  name="password"
                  type={show ? 'text' : 'password'}
                  required
                  autoComplete="current-password"
                  className="w-full rounded-lg border bg-white py-2.5 pr-10 pl-9"
                />
                <button
                  type="button"
                  onClick={() => setShow(!show)}
                  className="absolute top-1/2 right-2 -translate-y-1/2 rounded-md p-1 text-slate-400 hover:text-slate-600"
                  aria-label={show ? t('login.hide') : t('login.show')}
                >
                  {show ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                </button>
              </span>
            </label>
            {error && <p className="rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700">{error}</p>}
            <button disabled={pending} className="btn btn-primary w-full py-2.5">
              {pending ? t('common.loading') : t('auth.login')}
            </button>
          </form>
          <button
            onClick={() => i18n.changeLanguage(i18n.language === 'bn' ? 'en' : 'bn')}
            className="mx-auto mt-8 flex items-center gap-2 text-sm text-slate-500 hover:text-slate-800"
          >
            <Languages className="h-4 w-4" />
            {i18n.language === 'bn' ? 'English' : 'বাংলা'}
          </button>
        </div>
      </section>
    </div>
  )
}
