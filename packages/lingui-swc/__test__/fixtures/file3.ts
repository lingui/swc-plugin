import {t} from '@lingui/core/macro';

type User = {
  name: string;
  age: number;
};

const messages = {
  error: t`An error occurred`,
  success: t`Operation completed successfully`,
};
