import {extractMessages} from '../index'
import {expect, test} from 'vitest'

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

test('extractMessagesFromFiles: should extract from multiple files', async () => {
  const {extractMessagesFromFiles} = await import('../index')
  const path = await import('path')

  const filePaths = [
    path.join(__dirname, 'fixtures/file1.js'),
    path.join(__dirname, 'fixtures/file2.tsx'),
    path.join(__dirname, 'fixtures/file3.ts'),
  ]

  const result = await extractMessagesFromFiles(filePaths, {
    parser: {tsx: true, syntax: "typescript"}
  })

  // Sort messages by id for deterministic results
  result.messages.sort((a, b) => a.id.localeCompare(b.id))

  expect(result.warnings).toHaveLength(0)
  expect(result.messages).toHaveLength(6)
  expect(result.messages).toMatchSnapshot()
})

test('extractMessagesFromFiles: should handle non-existent files gracefully', async () => {
  const {extractMessagesFromFiles} = await import('../index')
  const path = await import('path')

  const filePaths = [
    path.join(__dirname, 'fixtures/file1.js'),
    path.join(__dirname, 'fixtures/non-existent.js'),
    path.join(__dirname, 'fixtures/file3.ts'),
  ]

  const result = await extractMessagesFromFiles(filePaths, {
    parser: {tsx: true, syntax: "typescript"}
  })

  // Sort messages by id for deterministic results
  result.messages.sort((a, b) => a.id.localeCompare(b.id))

  expect(result.warnings.length).toBeGreaterThan(0)
  expect(result.warnings.some(w => w.includes('non-existent.js'))).toBe(true)
  expect(result.messages.length).toBeGreaterThan(0) // Should still have messages from valid files
  expect(result.messages).toMatchSnapshot()
})

test('extractMessagesFromFiles: should handle empty array', async () => {
  const {extractMessagesFromFiles} = await import('../index')

  const result = await extractMessagesFromFiles([])

  expect(result.messages).toHaveLength(0)
  expect(result.warnings).toHaveLength(0)
})

test('extractMessagesFromFiles: should work with different parser options', async () => {
  const {extractMessagesFromFiles} = await import('../index')
  const path = await import('path')

  const filePaths = [
    path.join(__dirname, 'fixtures/file1.js'),
  ]

  const result = await extractMessagesFromFiles(filePaths, {
    parser: {syntax: "ecmascript"}
  })

  expect(result.messages.length).toBeGreaterThan(0)
  expect(result.warnings).toHaveLength(0)
})

test('extractMessagesFromFiles: messages should include origin with filename', async () => {
  const {extractMessagesFromFiles} = await import('../index')
  const path = await import('path')

  const filePaths = [
    path.join(__dirname, 'fixtures/file1.js'),
    path.join(__dirname, 'fixtures/file3.ts'),
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

  expect(result.messages).toMatchSnapshot()
})
