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

  describe('strips typescript syntax', () => {
    test('`as` expression in member object', async () => {
      const result = await transform(`(window as any).foo()`, 'app.tsx')

      expect(result.code).toMatchInlineSnapshot(`
        "window.foo();
        "
      `)
    })

    test('generic arrow function in .tsx', async () => {
      const result = await transform(`const f = <T,>(v: T) => v`, 'app.tsx')

      expect(result.code).toMatchInlineSnapshot(`
        "const f = (v)=>v;
        "
      `)
    })

    test('`as` expression in macro placeholder', async () => {
      const code = `
import { t } from '@lingui/core/macro';
const a = t\`Hello \${(user as any).name}\`;
`
      const result = await transform(code, 'app.ts')

      expect(result.code).toMatchInlineSnapshot(`
        "import { i18n as $_i18n } from "@lingui/core";
        const a = $_i18n._(/*i18n*/ {
            id: "Y7riaK",
            message: "Hello {0}",
            values: {
                0: user.name
            }
        });
        "
      `)
    })

    test('keeps macro imports used in JSX and via useLingui in .tsx', async () => {
      const code = `
import { Trans, useLingui } from '@lingui/react/macro';
import { msg } from '@lingui/core/macro';
import type { MessageDescriptor } from '@lingui/core';
const greeting: MessageDescriptor = msg\`Hello\`;
const App = ({ name }: { name: string }) => {
  const { t } = useLingui();
  return <div title={t\`Title\`}><Trans>Hello {name}</Trans></div>;
};
`
      const result = await transform(code, 'app.tsx')

      expect(result.code).toMatchInlineSnapshot(`
        "import { useLingui as $_useLingui } from "@lingui/react";
        import { Trans as Trans_ } from "@lingui/react";
        const greeting = /*i18n*/ {
            id: "uzTaYi",
            message: "Hello"
        };
        const App = ({ name })=>{
            const { i18n: $__i18n, _: $__ } = $_useLingui();
            return <div title={$__i18n._(/*i18n*/ {
                id: "MHrjPM",
                message: "Title"
            })}><Trans_ {.../*i18n*/ {
                id: "OVaF9k",
                values: {
                    name: name
                },
                message: "Hello {name}"
            }}/></div>;
        };
        "
      `)
    })

    test('removes type-only imports and keeps React import in .tsx', async () => {
      const code = `
import React from 'react';
import type { FC } from 'react';
import { Props } from './types';
const App: FC<Props> = () => <div />;
`
      const result = await transform(code, 'app.tsx')

      expect(result.code).toMatchInlineSnapshot(`
        "import React from 'react';
        const App = ()=><div/>;
        "
      `)
    })
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
      const Greet = (props)=><Trans_ {.../*i18n*/ {
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


  test('sourceMaps: true returns map in result', async () => {
    const code = `
import { t } from '@lingui/core/macro';
const msg = t\`Hello\`;
`
    const result = await transform(code, 'test.ts', {sourceMaps: true})

    expect(result.map).toBeDefined()
    const map = JSON.parse(result.map!)
    expect(map.version).toBe(3)
    expect(result.code).not.toContain('sourceMappingURL')
  })

  test('sourceMaps: false disables source maps', async () => {
    const code = `
import { t } from '@lingui/core/macro';
const msg = t\`Hello\`;
`
    const result = await transform(code, 'test.ts', {sourceMaps: false})

    expect(result.map).toBeUndefined()
    expect(result.code).not.toContain('sourceMappingURL')
  })

  test('sourceMaps: "inline" appends source map to code', async () => {
    const code = `
import { t } from '@lingui/core/macro';
const msg = t\`Hello\`;
`
    const result = await transform(code, 'test.ts', {sourceMaps: "inline"})

    expect(result.map).toBeUndefined()
    expect(result.code).toContain('//# sourceMappingURL=data:application/json;charset=utf-8;base64,')

    // extract and verify the inline map
    const match = result.code.match(/sourceMappingURL=data:application\/json;charset=utf-8;base64,(.+)/)
    expect(match).toBeTruthy()
    const decoded = JSON.parse(Buffer.from(match![1], 'base64').toString())
    expect(decoded.version).toBe(3)
    expect(decoded.sources).toContain('test.ts')
  })

  test('throws on parse errors', async () => {
    const code = 'const x = {'

    await expect(transform(code, 'broken.ts'))
      .rejects.toThrowError('Parse error')
  })
})
