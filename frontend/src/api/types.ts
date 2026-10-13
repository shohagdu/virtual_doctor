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

export type RxListKey = 'complaints' | 'examination' | 'investigations' | 'diagnosis' | 'advice' | 'next_plan' | 'chronic_conditions'

export interface ClinicSettings {
  rx_lists: Record<RxListKey, string[]>
  ai: { enabled: boolean; auto_extract: boolean; advice_language: 'en' | 'bn' | 'both' }
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

// ---- Phase 4: AI report analysis, summary, search ----

export type LabStatus = 'normal' | 'borderline' | 'high' | 'low' | 'critical' | 'abnormal'
export type ReportStatus = 'uploaded' | 'extracting' | 'extracted' | 'verified' | 'failed'
export type Confidence = 'high' | 'medium' | 'low'

export interface ReportResult {
  id: number
  test_code: string
  test_name: string
  value_text: string
  value_num: number | null
  unit: string | null
  ref_range_text: string | null
  lab_flag: string | null
  range_source: 'lab' | 'system' | null
  status: LabStatus | null
  confidence: Confidence | null
  edited_by_doctor: boolean
}

export interface Report {
  id: number
  patient_id: number
  visit_id: number | null
  report_type: string | null
  lab_name: string | null
  report_date: string | null
  mime_type: string
  original_name: string | null
  status: ReportStatus
  impression: string | null
  name_mismatch: boolean
  unreadable_parts: string[]
  error: string | null
  created_at: string
  results: ReportResult[]
}

export interface ResultInput {
  test_name: string
  value_text: string
  unit: string | null
  ref_range_text: string | null
  lab_flag: string | null
  confidence: Confidence | null
  edited: boolean
}

export type SuggestionSection = 'diagnosis' | 'investigation' | 'advice' | 'next_plan' | 'red_flag' | 'safety'

export interface Suggestion {
  id: number
  section: SuggestionSection
  text_en: string
  text_bn: string | null
  likelihood: string | null
  source_refs: string[]
  action: 'pending' | 'accepted' | 'edited' | 'rejected'
  edited_text: string | null
  reason: string | null
}

export interface ClinicalAnalysis {
  summary: string
  report_summaries: { report: string; summary: string }[]
  findings: { text: string; source: string[] }[]
  possible_issues: { name: string; reason: string; likelihood: string | null }[]
  suggested_investigations: { name: string; reason: string }[]
  advice_en: string[]
  advice_bn: string[]
  next_plan: string | null
  red_flags: string[]
  conflicts: string[]
  trend_comments: string[]
}

export interface AnalysisView {
  id: number
  content: ClinicalAnalysis
  claude_model: string
  prompt_version: string
  created_at: string
  suggestions: Suggestion[]
}

export interface SafetyWarning {
  id: number
  medicines: string[]
  kind: string
  severity: 'high' | 'moderate' | 'low' | string
  message: string
  source: string[]
  origin: 'rules' | 'ai'
}

export interface PatientSummary {
  one_line: string
  active_problems: string[]
  recurring_issues: string[]
  lab_trends: string[]
  medicines_history: string[]
  alerts: string[]
  points_to_check: string[]
}

export interface SavedSummary {
  id: number
  content: PatientSummary
  claude_model: string
  prompt_version: string
  created_at: string
}

export interface SearchFilters {
  sex?: 'M' | 'F'
  age_min?: number
  age_max?: number
  diagnosis_terms?: string[]
  complaint_terms?: string[]
  medicine_terms?: string[]
  chronic_terms?: string[]
  visited_from?: string
  visited_to?: string
}

export interface AiUsageRow {
  day: string
  analysis_type: string
  claude_model: string
  calls: number
  failed: number
  input_tokens: number
  output_tokens: number
  cache_read_tokens: number
}

// ---- Remaining items: templates, favourites, documents, merge ----

export interface RxTemplate {
  id: number
  name: string
  body: PrescriptionBody
  updated_at: string
}

export interface MedicineFavorite {
  id: number
  line: MedicineLine
}

export type DocumentKind = 'sick_leave' | 'fitness' | 'referral'

export interface DocumentBody {
  diagnosis: string | null
  from: string | null
  to: string | null
  referred_to: string | null
  reason: string | null
  summary: string | null
  remarks: string | null
}

export interface ClinicDocument {
  id: number
  visit_id: number
  patient_id: number
  kind: DocumentKind
  body: DocumentBody
  created_at: string
}

export interface MergeSide {
  patient: Patient
  counts: { visits: number; reports: number; appointments: number; bills: number }
  visits: VisitSummary[]
}
