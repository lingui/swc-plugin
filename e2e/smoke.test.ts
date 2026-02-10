import {describe, expect, it} from 'vitest'
import {transformFile} from '@swc/core'
import {resolve} from 'path'

const wasmPath = resolve(import.meta.dirname, '../target/wasm32-wasip1/release/lingui_macro_plugin.wasm')

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
})
