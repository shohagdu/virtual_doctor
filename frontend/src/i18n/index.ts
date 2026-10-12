import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import en from './en.json'
import bn from './bn.json'

const saved = (() => {
  try {
    return localStorage.getItem('lang')
  } catch {
    return null
  }
})()

i18n.use(initReactI18next).init({
  resources: { en: { translation: en }, bn: { translation: bn } },
  lng: saved ?? 'en',
  fallbackLng: 'en',
  interpolation: { escapeValue: false },
})

i18n.on('languageChanged', (lng) => {
  try {
    localStorage.setItem('lang', lng)
  } catch {
    /* storage unavailable */
  }
})

export default i18n
