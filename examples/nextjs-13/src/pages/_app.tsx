import '@/styles/globals.css';
import type { AppProps } from 'next/app';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
// import { useEffect, useRef } from 'react';

export default function MyApp({ Component, pageProps, router }: AppProps) {
  const locale = router.locale || router.defaultLocale!
  const messages = pageProps.i18n;

  // attempt 1
  // open in browser and try to change the language
  // Warning: Cannot update a component (`I18nProvider`) while rendering a different component (`MyApp`). To locate the bad setState() call inside `MyApp`
  // ====
  i18n.load(locale, messages);
  i18n.activate(locale);

  // attempt 2
  // this would simply don't work in SSR, because of useEffect
  // ====
  // useEffect(() => {
  //   i18n.load(locale, messages);
  //   i18n.activate(locale);
  // }, [locale, pageProps.i18n])
  // ====

  // attempt 3
  // this would work both in SSR and no issue in runtime but very verbose
  // ====
  // const firstRender = useRef(true)
  //
  // // run only once on the first render (for server side)
  // if (messages && firstRender.current) {
  //   i18n.load(locale, messages)
  //   i18n.activate(locale)
  //   firstRender.current = false
  // }
  //
  // useEffect(() => {
  //   if (messages) {
  //     i18n.load(locale, messages)
  //     i18n.activate(locale)
  //   }
  // }, [locale, messages])
  // ====

  // attempt 4 (current)
  // just set data without calling an event
  // ====
  // i18n.loadAndActivate(locale, messages, false);
  // ====

  return (
    <I18nProvider i18n={i18n}>
      <Component {...pageProps} />
    </I18nProvider>
  );
}
