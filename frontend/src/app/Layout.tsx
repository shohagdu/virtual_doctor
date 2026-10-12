import { NavLink, Outlet } from 'react-router'
import { useTranslation } from 'react-i18next'
import type { Role } from '../api/types'
import { useAuth } from '../features/auth/authStore'

const links: { to: string; key: string; roles?: Role[] }[] = [
  { to: '/dashboard', key: 'nav.dashboard' },
  { to: '/patients', key: 'nav.patients' },
  { to: '/appointments', key: 'nav.appointments' },
  { to: '/followups', key: 'nav.followups' },
  { to: '/collection', key: 'nav.collection', roles: ['doctor', 'admin'] },
  { to: '/settings', key: 'nav.settings' },
]

export function Layout() {
  const { t, i18n } = useTranslation()
  const { user, logout } = useAuth()

  const toggleLanguage = () => i18n.changeLanguage(i18n.language === 'bn' ? 'en' : 'bn')

  return (
    <div className="min-h-screen">
      <header className="bg-white shadow-sm print:hidden">
        <div className="mx-auto flex max-w-7xl flex-wrap items-center gap-4 px-4 py-3">
          <span className="text-lg font-semibold text-brand">{t('app.name')}</span>
          <nav className="flex flex-wrap gap-1">
            {links.filter((l) => !l.roles || (user && l.roles.includes(user.role))).map((l) => (
              <NavLink
                key={l.to}
                to={l.to}
                className={({ isActive }) =>
                  `rounded px-3 py-1.5 text-sm ${isActive ? 'bg-brand text-white' : 'text-slate-600 hover:bg-slate-100'}`
                }
              >
                {t(l.key)}
              </NavLink>
            ))}
          </nav>
          <div className="ml-auto flex items-center gap-3 text-sm">
            <button onClick={toggleLanguage} className="rounded border px-2 py-1">
              {i18n.language === 'bn' ? 'English' : 'বাংলা'}
            </button>
            <span className="text-slate-500">{user?.name}</span>
            <button onClick={logout} className="rounded bg-slate-500 px-3 py-1 text-white">
              {t('auth.logout')}
            </button>
          </div>
        </div>
      </header>
      <main className="mx-auto max-w-7xl p-4 print:max-w-none print:p-0">
        <Outlet />
      </main>
    </div>
  )
}
