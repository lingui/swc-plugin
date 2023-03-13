import { i18n } from '@lingui/core';
import { en, cs } from 'make-plural/plurals';

export const locales = [
  { twoLettersCode: 'en', label: 'English' },
  { twoLettersCode: 'cs', label: 'Česky' },
];


i18n.loadLocaleData({
  en: { plurals: en },
  cs: { plurals: cs },
});

export async function loadCatalog(locale: string) {
  const { messages } = await import(`@lingui/loader!../locales/${locale}/messages.po`);
  return messages;
}

// If not we can just load all the catalogs and do a simple i18n.active(localeToActive)
// i18n.load({
//   en: messagesEn,
//   cs: messagesCs,
// });
