import {extractMessages, extractMessagesFromFiles} from '../src-js/index'
import {describe, expect, test} from 'vitest'
import path from 'path'

test('test bindings', async () => {
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


test('parser options: should parse jsx', async () => {
  const code = `const t = <div>Hello</div>`

  await extractMessages(code, 'test.js', {
    parser: {jsx: true, syntax: "ecmascript"}
  })
})

test('parser options: should parse typescript', async () => {
  const code = `type Test = number; const t: Test = 5;`

  await extractMessages(code, 'test.js', {
    parser: {tsx: false, syntax: "typescript",}
  })

})

describe('extractMessagesFromFiles', () => {
  test('should extract from multiple files', async () => {
    const filePaths = [
      path.join(import.meta.dirname, 'fixtures/file1.js'),
      path.join(import.meta.dirname, 'fixtures/file2.tsx'),
      path.join(import.meta.dirname, 'fixtures/file3.ts'),
    ]

    const result = await extractMessagesFromFiles(filePaths, {
      parser: {tsx: true, syntax: "typescript"}
    })

    // Sort messages by id for deterministic results
    result.messages.sort((a, b) => a.id.localeCompare(b.id))

    result.messages.forEach((msg) => {
      msg.origin!.filename = path.relative(path.join(import.meta.dirname, 'fixtures'), msg.origin?.filename!)
    })

    expect(result.warnings).toHaveLength(0)
    expect(result.messages).toHaveLength(6)
    expect(result.messages).toMatchSnapshot()
  })

  test('should handle non-existent files gracefully', async () => {
    const filePaths = [
      path.join(import.meta.dirname, 'fixtures/file1.js'),
      path.join(import.meta.dirname, 'fixtures/non-existent.js'),
      path.join(import.meta.dirname, 'fixtures/file3.ts'),
    ]

    const result = await extractMessagesFromFiles(filePaths, {
      parser: {tsx: true, syntax: "typescript"}
    })

    // Sort messages by id for deterministic results
    result.messages.sort((a, b) => a.id.localeCompare(b.id))

    expect(result.warnings.length).toBeGreaterThan(0)
    expect(result.warnings.some(w => w.includes('non-existent.js'))).toBe(true)
    expect(result.messages.length).toBeGreaterThan(0) // Should still have messages from valid files
  })

  test('should handle empty array', async () => {
    const result = await extractMessagesFromFiles([])

    expect(result.messages).toHaveLength(0)
    expect(result.warnings).toHaveLength(0)
  })

  test('should work with different parser options', async () => {
    const filePaths = [
      path.join(import.meta.dirname, 'fixtures/file1.js'),
    ]

    const result = await extractMessagesFromFiles(filePaths, {
      parser: {syntax: "ecmascript"}
    })

    expect(result.messages.length).toBeGreaterThan(0)
    expect(result.warnings).toHaveLength(0)
  })

  test('messages should include origin with filename', async () => {
    const filePaths = [
      path.join(import.meta.dirname, 'fixtures/file1.js'),
      path.join(import.meta.dirname, 'fixtures/file3.ts'),
    ]

    const result = await extractMessagesFromFiles(filePaths, {
      parser: {syntax: "typescript"}
    })

    // Sort messages by id for deterministic results
    result.messages.sort((a, b) => a.id.localeCompare(b.id))

    expect(result.messages.length).toBeGreaterThan(0)

    // Check that all messages have origin with filename
    result.messages.forEach(msg => {
      expect(msg.origin).toBeDefined()
      expect(msg.origin?.filename).toBeDefined()
      expect(msg.origin?.filename).toMatch(/file[13]\.(js|ts)$/)
    })
  })
})

