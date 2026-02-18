// Configuração i18n para todos os frontends
import i18n from 'react-i18next'

// Definir idioma padrão como português
i18n.changeLanguage('pt')

// Salvar preferência no localStorage
if (typeof window !== 'undefined') {
  localStorage.setItem('i18nextLng', 'pt')
}

// Para uso em outros componentes
export const useTranslation = () => {
  const { t } = useTranslation()
  return t
}

// Hook para garantir português em desenvolvimento
export const usePortuguese = () => {
  const { i18n } = useTranslation()
  
  // Forçar português como padrão em ambiente de desenvolvimento
  if (process.env.NODE_ENV !== 'production') {
    i18n.changeLanguage('pt')
  }
  
  return i18n
}

export default useTranslation
