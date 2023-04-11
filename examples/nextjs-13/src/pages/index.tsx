import React, { useState } from 'react';
import { Trans, Plural } from '@lingui/macro';
import { i18n } from '@lingui/core';

import { loadCatalog } from '../i18n';
import { GetServerSidePropsContext, GetServerSidePropsResult } from 'next';
import { useLingui } from '@lingui/react';
import { LocaleButtons } from '@/pages/LocaleButtons';

export async function getServerSideProps(
  ctx: GetServerSidePropsContext
): Promise<GetServerSidePropsResult<any>> {
  return {
    props: {
      // we need to pass catalog to the client side to be able to **synchronously** consume it
      // on hydration phase. Otherwise, hydration mismatch would happend.
      i18n: await loadCatalog(ctx.locale as string),
    },
  };
}

const LinguiLocaleIndicator = React.memo(() => {
  const { i18n } = useLingui();
  return <>locale: {i18n.locale}</>;
});

function Home() {
  const [count, setCount] = useState(0);

  return (
    <div className="App">
      <header className="App-header">
        <img className="App-logo" src="https://avatars3.githubusercontent.com/u/11225539?s=200&v=4" />
        <h3 data-testid="h3-title">
          <Trans>Language switcher example: </Trans>
        </h3>
        <LocaleButtons />
        <h3>
          <Trans>Plurals example: </Trans>
        </h3>
        <div className="lang-container">
          <button type="button" onClick={() => setCount((state) => state + 1)}>
            <Trans>Increment</Trans>
          </button>
          <button type="button" onClick={() => setCount((state) => state - 1)}>
            <Trans>Decrement</Trans>
          </button>
        </div>
        <Plural
          value={count}
          zero={'There are no books'}
          one={"There's one book"}
          other={'There are # books'}
        />
        <h3>
          <Trans>Date formatter example:</Trans>
        </h3>
        <div>
          <Trans>Today is {i18n.date(new Date(), {})}</Trans>
        </div>
        <h3>
          <Trans>Number formatter example:</Trans>
        </h3>
        <div>
          <Trans>
            I have a balance of {i18n.number(1_000_000, { style: 'currency', currency: 'EUR' })}
          </Trans>
        </div>
        <LinguiLocaleIndicator />
      </header>
    </div>
  );
}

const MemoedHome = React.memo(Home);

export default function MemoedHomeWrapper() {
  // passing no props to MemoedHome so that it won't re-render
  // you can export Home directly but the LinguiLocaleIndicator will still be broken
  return <MemoedHome />;
}
