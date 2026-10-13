import type { ComponentType, ReactNode } from 'react'
import { Link } from 'react-router'

const tones = {
  blue: 'bg-brand-50 text-brand',
  amber: 'bg-amber-50 text-amber-600',
  green: 'bg-emerald-50 text-emerald-600',
  violet: 'bg-violet-50 text-violet-600',
  rose: 'bg-rose-50 text-rose-600',
  slate: 'bg-slate-100 text-slate-600',
}

/** A headline number with an icon (dashboard, collection, AI usage). */
export function StatCard({
  label,
  value,
  icon: Icon,
  tone = 'blue',
  hint,
  to,
}: {
  label: ReactNode
  value: ReactNode
  icon: ComponentType<{ className?: string }>
  tone?: keyof typeof tones
  hint?: ReactNode
  to?: string
}) {
  const body = (
    <>
      <span className={`flex h-11 w-11 shrink-0 items-center justify-center rounded-xl ${tones[tone]}`}>
        <Icon className="h-5 w-5" />
      </span>
      <span className="min-w-0">
        <span className="block truncate text-sm text-slate-500">{label}</span>
        <span className="block text-2xl leading-tight font-bold text-slate-900 tabular-nums">{value}</span>
        {hint && <span className="block truncate text-xs text-slate-500">{hint}</span>}
      </span>
    </>
  )
  const cls = 'flex items-center gap-4 rounded-2xl border border-slate-200/80 bg-white p-4 shadow-sm'
  return to ? (
    <Link to={to} className={`${cls} hover:border-brand/40 hover:shadow-md`}>
      {body}
    </Link>
  ) : (
    <div className={cls}>{body}</div>
  )
}
