import React from 'react';
import { locales } from '@/i18n';
import { useRouter } from 'next/router';

export const LocaleButtons = () => {
  const router = useRouter();

  return (
    <div className="lang-container">
      {locales.map((locale) => (
        <button
          type="button"
          onClick={() => router.push('/', '/', { locale: locale.twoLettersCode })}
          key={locale.twoLettersCode}
        >
          {locale.label}
        </button>
      ))}
    </div>
  );
};
