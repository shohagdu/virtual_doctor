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
    medicines: MedicineLine[]
    next_visit_date: string | null
  } | null
}

export interface VisitContext {
  visit: Visit
  patient: Patient
  profile: MedicalProfile
  previous_visit_date: string | null
  vitals: Vitals | null
}

/** BP mmHg, pulse /min, temp °F, weight kg, height cm, SpO₂ %, RBS mmol/L. */
export interface Vitals {
  bp_sys: number | null
  bp_dia: number | null
  pulse: number | null
  temp: number | null
  weight: number | null
  height: number | null
  bmi: number | null
  spo2: number | null
  rbs: number | null
}

export interface VitalsRecord extends Vitals {
  visit_id: number
  visit_date: string
  recorded_at: string
}

/** The patient fields shown in lists (queue, appointments, follow-ups). */
export interface PatientBrief {
  id: number
  patient_code: string
  title: string | null
  name_en: string
  name_bn: string | null
  phone: string
  dob: string
  sex: Sex
}

export interface QueueEntry extends Visit {
  patient: PatientBrief
  has_allergy: boolean
  has_vitals: boolean
  bill_status: BillStatus | null
}

export interface QueueToday {
  counters: { today: number; waiting: number; in_consultation: number; seen: number; followups_due: number }
  items: QueueEntry[]
}

export interface AllergyMatch {
  brand: string
  allergy: string
  /** Shared drug class, or null for a direct name match. */
  class: string | null
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

// ---- Phase 3: settings, appointments, follow-ups, billing ----

export type Lang = 'en' | 'bn'

export interface ChamberSession {
  /** 0 = Sunday … 6 = Saturday */
  weekday: number
  start: string
  end: string
}

export interface SmsTemplate {
  en: string
  bn: string
}

export interface ClinicSettings {
  fees: Record<VisitType, number>
  schedule: ChamberSession[]
  slot_minutes: number
  daily_limit: number | null
  sms: {
    enabled: boolean
    language: Lang
    confirm_on_book: boolean
    remind_day_before: boolean
    templates: {
      appointment_confirm: SmsTemplate
      appointment_reminder: SmsTemplate
      followup_reminder: SmsTemplate
    }
  }
}

export type AppointmentStatus = 'booked' | 'arrived' | 'seen' | 'no_show' | 'cancelled'

export interface Appointment {
  id: number
  start_at: string
  /** Clinic-local date (YYYY-MM-DD) and time (HH:MM). */
  date: string
  time: string
  visit_type: VisitType
  status: AppointmentStatus
  notes: string | null
  visit_id: number | null
  sms_sent_at: string | null
  patient: PatientBrief
}

export interface DaySlots {
  date: string
  free_time: boolean
  slots: { time: string; taken: boolean; past: boolean }[]
  booked: number
  daily_limit: number | null
}

export type FollowupBucket = 'today' | 'overdue' | 'week' | 'done'
export type FollowupStatus = 'pending' | 'came' | 'missed'

export interface Followup {
  id: number
  due_date: string
  status: FollowupStatus
  reminder_sent_at: string | null
  visit_id: number | null
  visit_date: string | null
  /** null for roles without clinical access */
  diagnosis: string[] | null
  chronic_conditions: string[]
  patient: PatientBrief
  last_sms: { id: number; status: 'queued' | 'sent' | 'failed'; error: string | null; created_at: string } | null
}

export interface FollowupList {
  counts: { today: number; overdue: number; week: number }
  items: Followup[]
}

export interface SmsDraft {
  body: string
  segments: number
  enabled: boolean
  phone_ok: boolean
}

export type PaymentMethod = 'cash' | 'bkash' | 'nagad' | 'card'
export type BillStatus = 'paid' | 'due'

export interface Bill {
  id: number
  visit_id: number
  receipt_no: string
  fee: number
  discount: number
  discount_reason: string | null
  total: number
  paid: number
  due: number
  method: PaymentMethod
  status: BillStatus
  created_at: string
  updated_at: string
}

export interface BillInput {
  fee: number
  discount: number
  discount_reason: string | null
  paid: number
  method: PaymentMethod
}

export interface CollectionRow {
  bill_id: number
  visit_id: number
  visit_date: string
  receipt_no: string
  patient_id: number
  patient_code: string
  patient_name: string
  visit_type: VisitType
  fee: number
  discount: number
  total: number
  paid: number
  due: number
  method: PaymentMethod
}

export interface Collection {
  from: string
  to: string
  totals: {
    count: number
    fee: number
    discount: number
    total: number
    paid: number
    due: number
    by_method: { method: PaymentMethod; paid: number; count: number }[]
  }
  rows: CollectionRow[]
}
