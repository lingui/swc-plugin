import { i18n, Messages } from '@lingui/core';
import { useRef, useEffect } from 'react';
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
  const firstRender = useRef(true)

  // run only once on the first render (for server side)
  if (messages && firstRender.current) {
    i18n.load(locale, messages)
    i18n.activate(locale)
    firstRender.current = false
  }

  useEffect(() => {
    if (messages) {
      i18n.load(locale, messages)
      i18n.activate(locale)
    }
  }, [locale, messages])

  return i18n;
}
