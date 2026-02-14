import {extractMessages} from '../index'
import {expect, test} from 'vitest'

test('test bindings', async (t) => {
  const code = `
  import {t} from '@lingui/core/macro';
  t\`Hello world\`;
  `

  const result = await extractMessages(code, 'test.js')

  // Check that we have no warnings
  // t.is(result.warnings.length, 0, `Expected no warnings, got: ${JSON.stringify(result.warnings)}`)

  expect(result).toMatchInlineSnapshot(`
    {
      "messages": [
        {
          "id": "1nGWAC",
          "message": "Hello world",
          "origin": {
            "column": 4,
            "filename": "test.js",
            "line": 3,
          },
          "placeholders": {},
        },
      ],
      "warnings": [],
    }
  `);
})


test('handle extraction errors gracefully', async () => {
  const invalidCode = 'const x = {'
  await expect(async () => await extractMessages(invalidCode, 'invalid.js'))
    .rejects
    .toThrowError("Parse error")
})


