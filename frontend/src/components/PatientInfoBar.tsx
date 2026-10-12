import type { ReactNode } from 'react'
import { Link } from 'react-router'
import type { Patient } from '../api/types'
import { ageText } from '../utils/age'

const sexLabel = { M: 'Male', F: 'Female', O: 'Other' } as const

/** The light-blue patient bar shared by the profile and prescription screens (§5.5). */
export function PatientInfoBar({
  patient: p,
  allergies,
  linkToProfile = false,
  children,
}: {
  patient: Patient
  allergies: string[]
  linkToProfile?: boolean
  children?: ReactNode
}) {
  const name = [p.title, p.name_en].filter(Boolean).join(' ')
  return (
    <div className="flex flex-wrap gap-x-6 gap-y-1 rounded-lg bg-infobar px-4 py-3 text-sm">
      <span>
        <b>ID:</b> {p.patient_code}
      </span>
      <span>
        <b>Name:</b> {linkToProfile ? <Link to={`/patients/${p.id}`} className="text-brand underline">{name}</Link> : name}
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
      {children}
      {allergies.length > 0 && (
        <span className="rounded bg-red-600 px-2 font-semibold text-white">⚠ Allergy: {allergies.join(', ')}</span>
      )}
    </div>
  )
}
