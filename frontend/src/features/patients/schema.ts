import { z } from 'zod'

export const patientSchema = z
  .object({
    title: z.string().optional(),
    name_en: z.string().trim().min(2, 'Name is required'),
    name_bn: z.string().optional(),
    phone: z.string().regex(/^01[3-9]\d{8}$/, 'Use Bangladesh format 01XXXXXXXXX'),
    dob: z.string().optional(),
    age_years: z.number({ error: 'Enter a number' }).int().min(0).max(130).optional(),
    sex: z.enum(['M', 'F', 'O']),
    blood_group: z.string().optional(),
    address: z.string().optional(),
    district: z.string().optional(),
  })
  .refine((v) => !!v.dob || v.age_years !== undefined, {
    message: 'Age or date of birth is required',
    path: ['age_years'],
  })

export type PatientForm = z.infer<typeof patientSchema>
