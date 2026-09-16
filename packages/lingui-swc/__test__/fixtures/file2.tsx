import {Trans} from '@lingui/react/macro';

function Component() {
  return (
    <div>
      <Trans id="welcome">Welcome to our app</Trans>
      <Trans id="user-greeting">Hello {userName}</Trans>
    </div>
  );
}
