import { useTranslation } from 'react-i18next'
import { ComingSoon } from '../../components/ComingSoon'

export function AppointmentsPage() {
  const { t } = useTranslation()
  return <ComingSoon title={t('nav.appointments')} />
}
