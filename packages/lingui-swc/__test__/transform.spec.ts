import {transform} from '../src-js/index'
import {describe, expect, test} from 'vitest'

describe('transform', () => {
  test('transforms t`` macro', async () => {
    const code = `
import { t } from '@lingui/core/macro';
const msg = t\`Hello world\`;
`
    const result = await transform(code, 'test.ts')

    expect(result.code).toMatchInlineSnapshot(`
      "import { i18n as $_i18n } from "@lingui/core";
      const msg = $_i18n._(/*i18n*/ {
          id: "1nGWAC",
          message: "Hello world"
      });
      "
    `)
    expect(result.map).toBeDefined()
  })

  test('transforms Trans JSX component', async () => {
    const code = `
import { Trans } from '@lingui/react/macro';
const App = () => <Trans>Hello world</Trans>;
`
    const result = await transform(code, 'app.tsx')

    expect(result.code).toMatchInlineSnapshot(`
      "import { Trans as Trans_ } from "@lingui/react";
      const App = ()=><Trans_ {.../*i18n*/ {
              id: "1nGWAC",
              message: "Hello world"
          }}/>;
      "
    `)
    expect(result.map).toBeDefined()
  })

  test('keeps non-lingui code unchanged', async () => {
    const code = `
import { useState } from 'react';
const App = () => {
  const [count, setCount] = useState(0);
  return <div>{count}</div>;
};
`
    const result = await transform(code, 'app.tsx')

    expect(result.code).toMatchInlineSnapshot(`
      "import { useState } from 'react';
      const App = ()=>{
          const [count, setCount] = useState(0);
          return <div>{count}</div>;
      };
      "
    `)
    expect(result.map).toBeDefined()
  })

  test('infers parser from .tsx filename', async () => {
    const code = `
import { Trans } from '@lingui/react/macro';
type Props = { name: string };
const Greet = (props: Props) => <Trans>Hello {props.name}</Trans>;
`
    const result = await transform(code, 'Greet.tsx')

    expect(result.code).toMatchInlineSnapshot(`
      "import { Trans as Trans_ } from "@lingui/react";
      type Props = {
          name: string;
      };
      const Greet = (props: Props)=><Trans_ {.../*i18n*/ {
              id: "Y7riaK",
              values: {
                  0: props.name
              },
              message: "Hello {0}"
          }}/>;
      "
    `)
    expect(result.code).not.toContain('@lingui/react/macro')
    expect(result.code).toContain('props.name')
  })

  test('infers parser from .js filename (no type annotations)', async () => {
    const code = `
import { t } from '@lingui/core/macro';
const msg = t\`Hello\`;
`
    const result = await transform(code, 'app.js')

    expect(result.code).not.toContain('@lingui/core/macro')
  })

  test('returns valid source map JSON', async () => {
    const code = `
import { t } from '@lingui/core/macro';
const msg = t\`Hello\`;
`
    const result = await transform(code, 'test.ts')

    expect(result.map).toBeDefined()
    const map = JSON.parse(result.map!)
    expect(map.version).toBe(3)
    expect(map.sources).toContain('test.ts')
    expect(map.sourcesContent).toBeDefined()
    expect(map.sourcesContent[0]).toBe(code)
  })

  test('handles inline source maps', async () => {
    const originalCode = `import { t } from '@lingui/core/macro';\nconst msg = t\`Hi\`;\n`
    const fakeMap = JSON.stringify({
      version: 3,
      sources: ['original.ts'],
      names: [],
      mappings: 'AAAA;AACA',
      sourcesContent: ['// original source']
    })
    const base64Map = Buffer.from(fakeMap).toString('base64')
    const codeWithInlineMap = originalCode + `//# sourceMappingURL=data:application/json;base64,${base64Map}\n`

    const result = await transform(codeWithInlineMap, 'test.ts')

    expect(result.code).not.toContain('@lingui/core/macro')
    expect(result.map).toBeDefined()
    const outputMap = JSON.parse(result.map!)
    expect(outputMap.sources).toContain('original.ts')
  })

  test('handles external source map passed in options', async () => {
    const code = `import { t } from '@lingui/core/macro';\nconst msg = t\`Hi\`;\n`
    const externalMap = JSON.stringify({
      version: 3,
      sources: ['src/original.ts'],
      names: [],
      mappings: 'AAAA;AACA',
      sourcesContent: ['// original source content']
    })

    const result = await transform(code, 'test.ts', {sourceMap: externalMap})

    expect(result.code).not.toContain('@lingui/core/macro')
    expect(result.map).toBeDefined()
    const outputMap = JSON.parse(result.map!)
    expect(outputMap.sources).toContain('src/original.ts')
  })

  test('throws on parse errors', async () => {
    const code = 'const x = {'

    await expect(transform(code, 'broken.ts'))
      .rejects.toThrowError('Parse error')
  })
})
