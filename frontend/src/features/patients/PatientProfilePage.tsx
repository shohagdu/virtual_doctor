import { useParams } from 'react-router'
import { useTranslation } from 'react-i18next'
import { Card } from '../../components/Card'
import { ageText } from '../../utils/age'
import { usePatient } from './api'

const sexLabel = { M: 'Male', F: 'Female', O: 'Other' } as const

const leftSections = [
  'Allergies',
  'Chronic Conditions',
  'Current Medicines',
  'Past History',
  'Family History',
  'Social History',
]

export function PatientProfilePage() {
  const { id = '' } = useParams()
  const { t } = useTranslation()
  const { data: p, isLoading } = usePatient(id)

  if (isLoading || !p) return <p className="text-slate-500">{t('common.loading')}</p>

  return (
    <div className="space-y-4">
      {/* Patient info bar — same light-blue bar as the prescription screen */}
      <div className="flex flex-wrap gap-x-6 gap-y-1 rounded-lg bg-infobar px-4 py-3 text-sm">
        <span>
          <b>ID:</b> {p.patient_code}
        </span>
        <span>
          <b>Name:</b> {[p.title, p.name_en].filter(Boolean).join(' ')}
        </span>
        <span>
          <b>Age:</b> {ageText(p.dob)}
          {p.dob_estimated && ' (est.)'}
        </span>
        <span>
          <b>Sex:</b> {sexLabel[p.sex]}
        </span>
        <span>
          <b>Phone:</b> {p.phone}
        </span>
        {p.blood_group && (
          <span>
            <b>Blood:</b> {p.blood_group}
          </span>
        )}
      </div>
      <div className="grid gap-4 md:grid-cols-[260px_1fr]">
        <aside className="space-y-3 rounded-lg bg-panel p-4">
          {leftSections.map((s) => (
            <div key={s} className="flex items-center justify-between text-sm font-semibold">
              {s} <span className="text-brand">➕</span>
            </div>
          ))}
        </aside>
        <Card title="Visits">
          <p className="text-slate-500">{t('common.comingSoon')}</p>
        </Card>
      </div>
    </div>
  )
}
