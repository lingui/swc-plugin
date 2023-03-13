import '@/styles/globals.css';
import type { AppProps } from 'next/app';
import { I18nProvider } from '@lingui/react';
import { useLinguiInit } from '../i18n';

export default function MyApp({ Component, pageProps, router }: AppProps) {
  const i18n = useLinguiInit(pageProps.i18n);

  return (
    <I18nProvider i18n={i18n}>
      <Component {...pageProps} />
    </I18nProvider>
  );
}
