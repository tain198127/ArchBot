// ═══════════════════════════════════════════════════════════════
// ExpressionEvaluator — L1 级简单表达式求值
//
// 支持: ==  !=  &&  ||  !  ( )
// 操作数: 布尔字面量 (true/false) | 一级属性访问 (project.loaded)
// 禁止: 方法调用、链式二级以上、函数调用
// ═══════════════════════════════════════════════════════════════

import type { RuntimeState, ContextObject } from './RuntimeContext'

/** Evaluate a simple boolean expression against runtime state */
export function evaluateExpression(
  expr: string | undefined,
  state: RuntimeState,
  context?: ContextObject,
): boolean {
  if (!expr || expr.trim() === '') return true

  const trimmed = expr.trim()

  try {
    const resolved = resolveProperties(trimmed, state, context)
    return parseExpression(resolved)
  } catch {
    return false
  }
}

function resolveProperties(
  expr: string,
  state: RuntimeState,
  context?: ContextObject,
): string {
  let result = expr.replace(/project\.(\w+)/g, (_, key: string) => {
    const s = state.project as unknown as Record<string, unknown>
    if (key in s) return JSON.stringify(s[key])
    return 'false'
  })

  if (context) {
    result = result.replace(/context\.(\w+)/g, (_, key: string) => {
      if (key === 'resourceType') return JSON.stringify(context.resourceType)
      if (key === 'semanticType') return JSON.stringify(context.semanticType ?? '')
      return 'false'
    })
  }

  return result
}

// ── Simple recursive-descent parser (no eval / no new Function) ──

type Token =
  | { type: 'lit'; value: boolean | string }
  | { type: 'op'; value: string }
  | { type: 'lparen' }
  | { type: 'rparen' }
  | { type: 'eof' }

class Lexer {
  private pos = 0
  private pushback: Token | null = null
  constructor(private input: string) {}

  next(): Token {
    if (this.pushback) { const t = this.pushback; this.pushback = null; return t }
    this.skipWhitespace()
    if (this.pos >= this.input.length) return { type: 'eof' }

    const ch = this.input[this.pos]

    if (ch === '(') { this.pos++; return { type: 'lparen' } }
    if (ch === ')') { this.pos++; return { type: 'rparen' } }

    if (ch === '!' && this.input[this.pos + 1] !== '=') {
      this.pos++; return { type: 'op', value: '!' }
    }
    if (ch === '&' && this.input[this.pos + 1] === '&') {
      this.pos += 2; return { type: 'op', value: '&&' }
    }
    if (ch === '|' && this.input[this.pos + 1] === '|') {
      this.pos += 2; return { type: 'op', value: '||' }
    }
    if (ch === '=' && this.input[this.pos + 1] === '=') {
      this.pos += 2; return { type: 'op', value: '==' }
    }
    if (ch === '!' && this.input[this.pos + 1] === '=') {
      this.pos += 2; return { type: 'op', value: '!=' }
    }

    if (ch === '"' || ch === "'") {
      const quote = ch; this.pos++
      let str = ''
      while (this.pos < this.input.length && this.input[this.pos] !== quote) {
        str += this.input[this.pos++]
      }
      this.pos++ // skip closing quote
      return { type: 'lit', value: str }
    }

    const word = this.readWord()
    if (word === 'true') return { type: 'lit', value: true }
    if (word === 'false') return { type: 'lit', value: false }
    throw new Error(`Unexpected token: ${word}`)
  }

  pushBack(t: Token): void {
    this.pushback = t
  }

  private skipWhitespace() {
    while (this.pos < this.input.length && /\s/.test(this.input[this.pos])) {
      this.pos++
    }
  }

  private readWord(): string {
    let word = ''
    while (this.pos < this.input.length && /\w/.test(this.input[this.pos])) {
      word += this.input[this.pos++]
    }
    return word
  }
}

function parseExpression(input: string): boolean {
  const lexer = new Lexer(input)
  return parseOr(lexer)
}

function parseOr(lexer: Lexer): boolean {
  let left = parseAnd(lexer)
  let token = lexer.next()
  while (token.type === 'op' && token.value === '||') {
    const right = parseAnd(lexer)
    left = left || right
    token = lexer.next()
  }
  lexer.pushBack(token)
  return left
}

function parseAnd(lexer: Lexer): boolean {
  let left = parseUnary(lexer)
  let token = lexer.next()
  while (token.type === 'op' && token.value === '&&') {
    const right = parseUnary(lexer)
    left = left && right
    token = lexer.next()
  }
  lexer.pushBack(token)
  return left
}

function parseUnary(lexer: Lexer): boolean {
  const token = lexer.next()
  if (token.type === 'op' && token.value === '!') {
    return !parseUnary(lexer)
  }
  lexer.pushBack(token)
  return parseComparison(lexer)
}

function parseComparison(lexer: Lexer): boolean {
  const left = parsePrimary(lexer)
  const token = lexer.next()

  if (token.type === 'op' && (token.value === '==' || token.value === '!=')) {
    const right = parsePrimary(lexer)
    const eq = token.value === '=='
    return eq ? left === right : left !== right
  }

  lexer.pushBack(token)
  return Boolean(left)
}

function parsePrimary(lexer: Lexer): unknown {
  const token = lexer.next()

  if (token.type === 'lparen') {
    const val = parseOr(lexer)
    const close = lexer.next()
    if (close.type !== 'rparen') throw new Error('Expected )')
    return val
  }

  if (token.type === 'lit') {
    return token.value
  }

  throw new Error(`Unexpected token: ${JSON.stringify(token)}`)
}
