import { useTranslation } from 'react-i18next'
import { Card } from './Card'

export function ComingSoon({ title }: { title: string }) {
  const { t } = useTranslation()
  return (
    <Card title={title}>
      <p className="text-slate-500">{t('common.comingSoon')}</p>
    </Card>
  )
}
