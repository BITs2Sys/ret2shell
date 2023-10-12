import i18next from 'i18next'
import { createI18nStore } from './store'
import LanguageDetector from 'i18next-browser-languagedetector'
import en_US from './en-us.json'
import zh_CN from './zh-cn.json'

i18next.use(LanguageDetector).init({
  fallbackLng: 'en_US',
  resources: {
    en_US: { translation: en_US },
    zh_CN: { translation: zh_CN },
  },
  interpolation: {
    escapeValue: false,
  },
})

export const i18n = createI18nStore(i18next)
