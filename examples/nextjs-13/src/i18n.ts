import { i18n, Messages } from '@lingui/core';
import { useRouter } from 'next/router';

export const locales = [
  { twoLettersCode: 'en', label: 'English' },
  { twoLettersCode: 'cs', label: 'Česky' },
];

export async function loadCatalog(locale: string) {
  const { messages } = await import(`@lingui/loader!../locales/${locale}/messages.po`);
  return messages;
}

export function useLinguiInit(messages: Messages) {
  const router = useRouter()
  const locale = router.locale || router.defaultLocale!
  i18n.loadAndActivate(locale, messages, false);

  return i18n;
}
