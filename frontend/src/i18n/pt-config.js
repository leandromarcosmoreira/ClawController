// Configuração para idioma português como padrão
import i18n from './index.js';

// Definir português como idioma padrão
i18n.changeLanguage('pt');

// Salvar preferência no localStorage
if (typeof window !== 'undefined') {
    localStorage.setItem('i18nextLng', 'pt');
}

export default i18n;
