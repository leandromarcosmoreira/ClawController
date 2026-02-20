import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';

// Import translation files
import en from './locales/en.json';
import es from './locales/es.json';
import pt from './locales/pt.json';

const resources = {
  en: {
    translation: en
  },
  es: {
    translation: es
  },
  pt: {
    translation: pt
  }
};

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources,
    fallbackLng: 'pt',
    lng: 'pt',
    debug: import.meta.env.DEV,
    
    interpolation: {
      escapeValue: false
    },

    detection: {
      order: ['localStorage', 'navigator', 'htmlTag'],
      caches: ['localStorage'],
      // Force Portuguese as default
      lookupLocalStorage: 'i18nextLng',
      checkWhitelist: true
    },
    
    react: {
      useSuspense: false
    }
  });

// Force Portuguese after init and override any browser detection
i18n.changeLanguage('pt');
localStorage.setItem('i18nextLng', 'pt');

// Additional force to ensure Portuguese is set
if (typeof window !== 'undefined') {
  window.i18n = i18n;
  // Force Portuguese on every load
  setTimeout(() => {
    i18n.changeLanguage('pt');
    localStorage.setItem('i18nextLng', 'pt');
  }, 100);
}

export default i18n;
