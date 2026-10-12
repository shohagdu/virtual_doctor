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
