import '@/styles/globals.css';
import type { AppProps } from 'next/app';
import { i18n } from '@lingui/core';
import { I18nProvider } from '@lingui/react';

export default function MyApp({ Component, pageProps, router }: AppProps) {
  // Warning! THis is not public api, I didn't find a way how to make it better for now.
  // Ususaly you have to call i18n.loadLocaleData() and i18n.activate()
  // But they both dispatch a `change` event which trigger a stateChange
  // and cause error in React because react still rendering current component
  i18n._messages[router.locale as string] = pageProps.i18n;
  i18n._locale = router.locale as string;

  // i18n.loadLocaleData(router.locale as string, pageProps.i18n);
  // i18n.activate(router.locale as string);

  return (
    <I18nProvider i18n={i18n}>
      <Component {...pageProps} />
    </I18nProvider>
  );
}
