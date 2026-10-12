import { useTranslation } from 'react-i18next'
import { Card } from '../../components/Card'

export function DashboardPage() {
  const { t } = useTranslation()
  // Counters and queue are wired up in Phase 2 (queue API).
  const counters = [
    { key: 'dashboard.today', value: '—' },
    { key: 'dashboard.waiting', value: '—' },
    { key: 'dashboard.seen', value: '—' },
    { key: 'dashboard.followupsDue', value: '—' },
  ]
  return (
    <div className="space-y-4">
      <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
        {counters.map((c) => (
          <div key={c.key} className="rounded-lg bg-white p-4 shadow-sm">
            <div className="text-sm text-slate-500">{t(c.key)}</div>
            <div className="text-2xl font-semibold">{c.value}</div>
          </div>
        ))}
      </div>
      <Card title={t('dashboard.queue')}>
        <p className="text-slate-500">{t('common.comingSoon')}</p>
      </Card>
    </div>
  )
}
