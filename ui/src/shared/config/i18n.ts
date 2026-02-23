import i18n from 'i18next'
import LanguageDetector from 'i18next-browser-languagedetector'
import HttpBackend from 'i18next-http-backend'
import { initReactI18next } from 'react-i18next'

void i18n
  // 1. Use the HTTP backend to load JSON files
  .use(HttpBackend)
  // 2. Use the browser language detector
  .use(LanguageDetector)
  // 3. Pass the i18n instance to react-i18next
  .use(initReactI18next)
  // 4. Initialize i18next
  .init({
    // The default language to fall back to
    fallbackLng: 'en',

    // Define the "namespaces" (translation files) your app will use.
    // 'common' is a good default for shared text (like "Save", "Cancel").
    ns: ['common'],
    defaultNS: 'common',

    // For development, set this to true to see logs
    debug: true,

    // Config for the HTTP backend
    backend: {
      // Path where your translation files will be.
      // /locales/{{lng}}/{{ns}}.json
      // e.g., /locales/en/common.json
      loadPath: '/locales/{{lng}}/{{ns}}.json',
    },

    interpolation: {
      escapeValue: false,
    },

    // React-specific configuration
    react: {
      // We use Suspense for lazy loading
      useSuspense: true,
    },
  })

export default i18n
