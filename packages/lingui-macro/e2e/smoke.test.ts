import {describe, expect, it} from 'vitest'
import {transform, transformFile} from '@swc/core'
import {resolve} from 'path'

const wasmPath = resolve(import.meta.dirname, '../lingui_macro.wasm')

async function transformWithSwc(filePath: string, envName?: string) {
  return transformFile(filePath, {
    filename: 'test.js',
    envName,
    jsc: {
      parser: {
        syntax: 'ecmascript',
        jsx: false,
      },
      experimental: {
        plugins: [
          [wasmPath, {}]
        ],
      },
    },
  })
}

describe('E2E Smoke Test', () => {
  it('should transform t macro using compiled WASM plugin', async () => {
    const fixturePath = resolve(import.meta.dirname, 'fixtures/simple-t-macro.js')
    const result = await transformWithSwc(fixturePath)
    expect(result.code).toMatchSnapshot()
  })

  it('should respect ENV production', async () => {
    const fixturePath = resolve(import.meta.dirname, 'fixtures/simple-t-macro.js')
    const result = await transformWithSwc(fixturePath, 'production')
    expect(result.code).toMatchSnapshot()
  })

  it('should keep lingui-set directives set on TypeScript export declarations', async () => {
    const result = await transform(
      `
        import { msg } from '@lingui/core/macro'

        // lingui-set context="navigation"
        type SomeType = string

        const table = {
          home: msg\`Home\`,
        }
      `,
      {
        filename: 'directive-after-export-type.ts',
        jsc: {
          parser: {
            syntax: 'typescript',
            tsx: false,
          },
          experimental: {
            plugins: [[wasmPath, { descriptorFields: 'message' }]],
          },
        },
      },
    )

    expect(result.code).toContain('context: "navigation"')
  })
})
