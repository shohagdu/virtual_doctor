import { useDeferredValue, useEffect, useRef, useState, type ComponentType } from 'react'
import { Link, NavLink, Outlet, useLocation, useNavigate } from 'react-router'
import { useTranslation } from 'react-i18next'
import { format } from 'date-fns'
import {
  BellRing,
  CalendarDays,
  ChevronDown,
  Languages,
  LayoutDashboard,
  LogOut,
  Menu,
  PanelLeftClose,
  PanelLeftOpen,
  Pill,
  Search,
  Settings,
  Stethoscope,
  Users,
  Wallet,
  X,
} from 'lucide-react'
import type { Role } from '../api/types'
import { useAuth } from '../features/auth/authStore'
import { usePatients } from '../features/patients/api'
import { useQueueToday } from '../features/queue/api'
import { ageText } from '../utils/age'
import { fullName } from '../utils/patient'

type Icon = ComponentType<{ className?: string; strokeWidth?: number }>

const links: { to: string; key: string; icon: Icon; roles?: Role[]; badge?: 'waiting' | 'followups' }[] = [
  { to: '/dashboard', key: 'nav.dashboard', icon: LayoutDashboard, badge: 'waiting' },
  { to: '/patients', key: 'nav.patients', icon: Users },
  { to: '/appointments', key: 'nav.appointments', icon: CalendarDays },
  { to: '/followups', key: 'nav.followups', icon: BellRing, badge: 'followups' },
  { to: '/medicines', key: 'nav.medicines', icon: Pill, roles: ['doctor', 'admin'] },
  { to: '/collection', key: 'nav.collection', icon: Wallet, roles: ['doctor', 'admin'] },
  { to: '/settings', key: 'nav.settings', icon: Settings },
]

function initials(name: string) {
  return name
    .replace(/^(dr|mr|mrs|ms)\.?\s+/i, '')
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((w) => w[0]?.toUpperCase())
    .join('')
}

/** App shell: sidebar navigation (a drawer on phones), top bar with quick patient search. */
export function Layout() {
  const { t } = useTranslation()
  const { user } = useAuth()
  const [menuOpen, setMenuOpen] = useState(false)
  const [today] = useState(() => format(new Date(), 'EEEE, d MMMM yyyy'))
  const location = useLocation()
  // The queue feeds the sidebar badges (waiting patients, follow-ups due).
  const { data: queue } = useQueueToday()
  const badges = { waiting: queue?.counters.waiting ?? 0, followups: queue?.counters.followups_due ?? 0 }
  // Pages that need room (patient profile, prescription) start with the sidebar
  // collapsed to icons; elsewhere the user's last choice is kept.
  const workspace = /^\/(patients|visits)\/\d+/.test(location.pathname)
  const [stored, setStored] = useState(() => readCollapsed())
  const [override, setOverride] = useState<boolean | null>(null)
  const [lastPath, setLastPath] = useState(location.pathname)
  if (lastPath !== location.pathname) {
    // Close the phone menu after navigating; a page's own default applies again.
    setLastPath(location.pathname)
    setMenuOpen(false)
    setOverride(null)
  }
  const collapsed = override ?? (workspace || stored)
  const toggle = () => {
    setOverride(!collapsed)
    if (!workspace) {
      setStored(!collapsed)
      writeCollapsed(!collapsed)
    }
  }

  const nav = (compact: boolean) => (
    <nav className={`flex flex-1 flex-col gap-1 py-4 ${compact ? 'px-2' : 'px-3'}`}>
      {links
        .filter((l) => !l.roles || (user && l.roles.includes(user.role)))
        .map(({ to, key, icon: Icon, badge }) => {
          const count = badge ? badges[badge] : 0
          const label = t(key)
          return (
            <NavLink
              key={to}
              to={to}
              title={compact ? label : undefined}
              aria-label={compact ? label : undefined}
              className={({ isActive }) =>
                `group relative flex items-center rounded-lg text-sm font-medium ${
                  compact ? 'justify-center p-2.5' : 'gap-3 px-3 py-2'
                } ${isActive ? 'bg-brand-50 text-brand' : 'text-slate-600 hover:bg-slate-100 hover:text-slate-900'}`
              }
            >
              <Icon className="h-[18px] w-[18px] shrink-0" strokeWidth={2} />
              {!compact && <span className="flex-1">{label}</span>}
              {count > 0 &&
                (compact ? (
                  <span
                    className={`absolute top-1 right-1 min-w-4 rounded-full px-1 text-center text-[10px] leading-4 font-bold ${
                      badge === 'followups' ? 'bg-amber-400 text-amber-950' : 'bg-brand text-white'
                    }`}
                  >
                    {count}
                  </span>
                ) : (
                  <span
                    className={`min-w-5 rounded-full px-1.5 text-center text-xs leading-5 font-semibold ${
                      badge === 'followups' ? 'bg-amber-100 text-amber-800' : 'bg-brand text-white'
                    }`}
                  >
                    {count}
                  </span>
                ))}
            </NavLink>
          )
        })}
    </nav>
  )

  const brand = (compact: boolean) => (
    <Link to="/dashboard" className={`flex items-center gap-2.5 py-4 ${compact ? 'justify-center px-2' : 'px-5'}`} title={t('app.name')}>
      <span className="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-gradient-to-br from-brand to-sky-500 text-white shadow-sm">
        <Stethoscope className="h-5 w-5" />
      </span>
      {!compact && (
        <span className="leading-tight">
          <span className="block text-[15px] font-bold text-slate-900">{t('app.name')}</span>
          <span className="block text-xs text-slate-500">{t('shell.tagline')}</span>
        </span>
      )}
    </Link>
  )

  return (
    <div className={`min-h-screen transition-[padding] duration-200 print:pl-0 ${collapsed ? 'lg:pl-[72px]' : 'lg:pl-64'}`}>
      {/* Sidebar: fixed on large screens (full or an icon rail), a drawer on small ones. */}
      <aside
        className={`fixed inset-y-0 left-0 z-20 hidden flex-col border-r border-slate-200 bg-white transition-[width] duration-200 lg:flex print:hidden ${
          collapsed ? 'w-[72px]' : 'w-64'
        }`}
      >
        {brand(collapsed)}
        {nav(collapsed)}
        <SidebarFooter compact={collapsed} onToggle={toggle} />
      </aside>
      {menuOpen && (
        <div className="fixed inset-0 z-40 lg:hidden print:hidden" role="dialog" aria-modal="true">
          <div className="absolute inset-0 bg-slate-900/40 backdrop-blur-[2px]" onClick={() => setMenuOpen(false)} />
          <aside className="absolute inset-y-0 left-0 flex w-72 flex-col bg-white shadow-2xl">
            <div className="flex items-center">
              {brand(false)}
              <button onClick={() => setMenuOpen(false)} className="ml-auto mr-3 rounded-lg p-2 hover:bg-slate-100" aria-label={t('common.close')}>
                <X className="h-5 w-5" />
              </button>
            </div>
            {nav(false)}
            <SidebarFooter compact={false} />
          </aside>
        </div>
      )}

      <header className="sticky top-0 z-10 border-b border-slate-200 bg-white/85 backdrop-blur print:hidden">
        <div className="flex items-center gap-3 px-4 py-2.5 lg:px-6">
          <button onClick={() => setMenuOpen(true)} className="rounded-lg p-2 hover:bg-slate-100 lg:hidden" aria-label={t('shell.menu')}>
            <Menu className="h-5 w-5" />
          </button>
          <QuickSearch />
          <span className="ml-auto hidden text-sm text-slate-500 md:block">{today}</span>
          <UserMenu />
        </div>
      </header>

      <main className={`mx-auto p-4 lg:p-6 print:max-w-none print:p-0 ${collapsed ? 'max-w-[1440px]' : 'max-w-7xl'}`}>
        <Outlet />
      </main>
    </div>
  )
}

const COLLAPSED_KEY = 'vd-sidebar-collapsed'

function readCollapsed() {
  try {
    return localStorage.getItem(COLLAPSED_KEY) === '1'
  } catch {
    return false
  }
}

function writeCollapsed(v: boolean) {
  try {
    localStorage.setItem(COLLAPSED_KEY, v ? '1' : '0')
  } catch {
    /* storage unavailable */
  }
}

function SidebarFooter({ compact, onToggle }: { compact: boolean; onToggle?: () => void }) {
  const { t, i18n } = useTranslation()
  const other = i18n.language === 'bn' ? 'English' : 'বাংলা'
  const row = `flex w-full items-center rounded-lg text-sm text-slate-600 hover:bg-slate-100 ${compact ? 'justify-center p-2.5' : 'gap-3 px-3 py-2'}`
  return (
    <div className={`space-y-1 border-t border-slate-200 ${compact ? 'p-2' : 'p-3'}`}>
      <button onClick={() => i18n.changeLanguage(i18n.language === 'bn' ? 'en' : 'bn')} className={row} title={compact ? other : undefined}>
        <Languages className="h-[18px] w-[18px]" />
        {!compact && other}
      </button>
      {onToggle && (
        <button
          onClick={onToggle}
          className={row}
          title={compact ? t('shell.expand') : t('shell.collapse')}
          aria-label={compact ? t('shell.expand') : t('shell.collapse')}
          aria-expanded={!compact}
        >
          {compact ? <PanelLeftOpen className="h-[18px] w-[18px]" /> : <PanelLeftClose className="h-[18px] w-[18px]" />}
          {!compact && t('shell.collapse')}
        </button>
      )}
      {!compact && <p className="px-3 pt-1 text-[11px] text-slate-400">{t('shell.version')}</p>}
    </div>
  )
}

function UserMenu() {
  const { t } = useTranslation()
  const { user, logout } = useAuth()
  const [open, setOpen] = useState(false)
  const ref = useRef<HTMLDivElement>(null)
  useEffect(() => {
    if (!open) return
    const close = (e: MouseEvent) => !ref.current?.contains(e.target as Node) && setOpen(false)
    document.addEventListener('mousedown', close)
    return () => document.removeEventListener('mousedown', close)
  }, [open])
  if (!user) return null
  return (
    <div ref={ref} className="relative">
      <button onClick={() => setOpen(!open)} className="flex items-center gap-2 rounded-full py-1 pr-2 pl-1 hover:bg-slate-100" aria-expanded={open}>
        <span className="flex h-8 w-8 items-center justify-center rounded-full bg-brand-100 text-xs font-bold text-brand">
          {initials(user.name)}
        </span>
        <span className="hidden text-left leading-tight sm:block">
          <span className="block text-sm font-medium text-slate-800">{user.name}</span>
          <span className="block text-xs text-slate-500">{t(`shell.role.${user.role}`)}</span>
        </span>
        <ChevronDown className="h-4 w-4 text-slate-400" />
      </button>
      {open && (
        <div className="absolute right-0 mt-2 w-52 overflow-hidden rounded-xl border border-slate-200 bg-white py-1 shadow-lg">
          <Link to="/settings" onClick={() => setOpen(false)} className="flex items-center gap-2 px-4 py-2 text-sm hover:bg-slate-50">
            <Settings className="h-4 w-4 text-slate-500" /> {t('nav.settings')}
          </Link>
          <button onClick={logout} className="flex w-full items-center gap-2 px-4 py-2 text-sm text-red-600 hover:bg-red-50">
            <LogOut className="h-4 w-4" /> {t('auth.logout')}
          </button>
        </div>
      )}
    </div>
  )
}

/** Find a patient from anywhere: name, phone or ID. Enter opens the full list. */
function QuickSearch() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const [q, setQ] = useState('')
  const [focus, setFocus] = useState(false)
  const query = useDeferredValue(q.trim())
  const { data } = usePatients(query, 1, query.length >= 2)
  const items = data?.items.slice(0, 6) ?? []
  const open = focus && query.length >= 2

  const go = (to: string) => {
    setQ('')
    setFocus(false)
    navigate(to)
  }

  return (
    <div className="relative w-full max-w-md">
      <Search className="pointer-events-none absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-slate-400" />
      <input
        value={q}
        onChange={(e) => setQ(e.target.value)}
        onFocus={() => setFocus(true)}
        onBlur={() => setTimeout(() => setFocus(false), 150)}
        onKeyDown={(e) => {
          if (e.key === 'Enter' && query) go(`/patients?q=${encodeURIComponent(query)}`)
          if (e.key === 'Escape') setQ('')
        }}
        placeholder={t('shell.search')}
        aria-label={t('shell.search')}
        className="w-full rounded-lg border bg-slate-50 py-2 pr-3 pl-9 text-sm focus:bg-white"
      />
      {open && (
        <div className="absolute top-full z-30 mt-1.5 w-full overflow-hidden rounded-xl border border-slate-200 bg-white shadow-lg">
          {items.length === 0 ? (
            <p className="px-4 py-3 text-sm text-slate-500">{data ? t('patients.empty') : t('common.loading')}</p>
          ) : (
            <ul className="divide-y divide-slate-100">
              {items.map((p) => (
                <li key={p.id}>
                  <button
                    onMouseDown={(e) => e.preventDefault()}
                    onClick={() => go(`/patients/${p.id}`)}
                    className="flex w-full items-center gap-3 px-4 py-2.5 text-left hover:bg-slate-50"
                  >
                    <span className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-slate-100 text-xs font-semibold text-slate-600">
                      {initials(p.name_en)}
                    </span>
                    <span className="min-w-0">
                      <span className="block truncate text-sm font-medium text-slate-800">{fullName(p)}</span>
                      <span className="block text-xs text-slate-500">
                        {p.patient_code} · {ageText(p.dob)} / {p.sex} · {p.phone}
                      </span>
                    </span>
                  </button>
                </li>
              ))}
            </ul>
          )}
          {data && data.total > items.length && (
            <button
              onMouseDown={(e) => e.preventDefault()}
              onClick={() => go(`/patients?q=${encodeURIComponent(query)}`)}
              className="block w-full border-t border-slate-100 px-4 py-2 text-left text-sm text-brand hover:bg-slate-50"
            >
              {t('shell.allResults', { count: data.total })} →
            </button>
          )}
        </div>
      )}
    </div>
  )
}
