export type Role = 'doctor' | 'assistant' | 'admin'
export type Sex = 'M' | 'F' | 'O'

export interface User {
  id: number
  name: string
  email: string
  role: Role
}

export interface LoginResponse {
  access_token: string
  user: User
}

export interface Patient {
  id: number
  patient_code: string
  title: string | null
  name_en: string
  name_bn: string | null
  phone: string
  dob: string
  dob_estimated: boolean
  sex: Sex
  blood_group: string | null
  address: string | null
  district: string | null
  created_at: string
}

export interface Page<T> {
  items: T[]
  total: number
  page: number
  per_page: number
}

export interface CreatePatientResponse {
  patient: Patient | null
  duplicates: Patient[]
}

export interface MedicalProfile {
  allergies: string[]
  chronic_conditions: string[]
  long_term_medicines: string[]
  vaccination: string[]
  past_history: string | null
  family_history: string | null
  social_history: string | null
  gynae_obs_history: string | null
  pregnancy_status: string | null
}

export type VisitType = 'new' | 'followup' | 'report'
export type VisitStatus = 'waiting' | 'in_consultation' | 'seen' | 'cancelled'

export interface Visit {
  id: number
  patient_id: number
  visit_no: number
  visit_date: string
  visit_type: VisitType
  status: VisitStatus
  queue_serial: number | null
  doctor_id: number | null
  created_at: string
}

export interface VisitSummary extends Visit {
  prescription: {
    version: number
    locked: boolean
    complaints: string[]
    diagnosis: string[]
    medicines: string[]
    next_visit_date: string | null
  } | null
}

export interface VisitContext {
  visit: Visit
  patient: Patient
  profile: MedicalProfile
  previous_visit_date: string | null
}

export interface MedicineLine {
  medicine_id: number | null
  brand: string
  generic: string | null
  strength: string | null
  form: string | null
  dose_pattern: string | null
  timing: string | null
  duration: string | null
  notes: string | null
}

export interface PrescriptionBody {
  complaints: string[]
  examination: string[]
  investigations: string[]
  diagnosis: string[]
  treatment_plan: string[]
  advice: string[]
  next_plan: string[]
  operation_notes: string | null
  next_visit_date: string | null
  medicines: MedicineLine[]
}

export interface Prescription extends PrescriptionBody {
  id: number | null
  visit_id: number
  version: number
  revision: number
  locked: boolean
  updated_at: string | null
}

export interface MedicineSuggestion {
  medicine_id: number | null
  brand: string
  generic: string | null
  strength: string | null
  form: string | null
}

export interface Me {
  user: {
    id: number
    name: string
    email: string
    phone: string | null
    role: Role
    degrees: string | null
    bmdc_reg_no: string | null
  }
  clinic: { name: string; address: string | null; phone: string | null }
}
