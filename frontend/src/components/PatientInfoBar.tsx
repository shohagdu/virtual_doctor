import type { ReactNode } from 'react'
import { Link } from 'react-router'
import { AlertTriangle, Droplet, Phone } from 'lucide-react'
import type { Patient } from '../api/types'
import { ageText } from '../utils/age'

const sexLabel = { M: 'Male', F: 'Female', O: 'Other' } as const

/** The patient bar shared by the profile and prescription screens (§5.5). */
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
  const letters = p.name_en
    .split(/\s+/)
    .slice(0, 2)
    .map((w) => w[0]?.toUpperCase())
    .join('')
  return (
    <div className="flex flex-wrap items-center gap-x-6 gap-y-3 rounded-2xl border border-brand/15 bg-gradient-to-r from-infobar to-white px-5 py-4 text-sm">
      <div className="flex items-center gap-3">
        <span
          className={`flex h-12 w-12 items-center justify-center rounded-full text-base font-bold ${
            p.sex === 'F' ? 'bg-pink-100 text-pink-700' : p.sex === 'M' ? 'bg-brand-100 text-brand' : 'bg-slate-200 text-slate-600'
          }`}
        >
          {letters}
        </span>
        <div>
          <div className="text-lg leading-tight font-semibold text-slate-900">
            {linkToProfile ? (
              <Link to={`/patients/${p.id}`} className="hover:text-brand hover:underline">
                {name}
              </Link>
            ) : (
              name
            )}
            {p.name_bn && <span className="ml-2 text-sm font-normal text-slate-500">{p.name_bn}</span>}
          </div>
          <div className="text-slate-600">
            {ageText(p.dob)}
            {p.dob_estimated && ' (est.)'} · {sexLabel[p.sex]} · <span className="font-mono text-xs">{p.patient_code}</span>
          </div>
        </div>
      </div>
      <span className="flex items-center gap-1.5 text-slate-700">
        <Phone className="h-4 w-4 text-slate-400" /> {p.phone}
      </span>
      {p.blood_group && (
        <span className="flex items-center gap-1.5 text-slate-700">
          <Droplet className="h-4 w-4 text-red-400" /> {p.blood_group}
        </span>
      )}
      {children}
      {allergies.length > 0 && (
        <span className="ml-auto inline-flex items-center gap-1.5 rounded-full bg-red-600 px-3 py-1 text-sm font-semibold text-white shadow-sm">
          <AlertTriangle className="h-4 w-4" /> Allergy: {allergies.join(', ')}
        </span>
      )}
    </div>
  )
}
