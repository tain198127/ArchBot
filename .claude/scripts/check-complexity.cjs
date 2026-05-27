#!/usr/bin/env node

/**
 * McCabe cyclomatic complexity checker
 *
 * Usage:
 *   node check-complexity.js <file-path> [function-name]
 *
 * Without function-name: reports all functions in the file
 * With function-name: reports only that function
 */

const fs = require('fs');
const path = require('path');

const filePath = process.argv[2];
const targetFunction = process.argv[3];

if (!filePath) {
  console.log('Usage: node check-complexity.js <file-path> [function-name]');
  process.exit(1);
}

const fullPath = path.resolve(filePath);
if (!fs.existsSync(fullPath)) {
  console.error(`File not found: ${fullPath}`);
  process.exit(1);
}

const content = fs.readFileSync(fullPath, 'utf-8');
const ext = path.extname(fullPath);

const isRust = ext === '.rs';
const isTS = ['.ts', '.tsx', '.js', '.jsx', '.vue'].includes(ext);

/**
 * Extract function blocks from source code
 *
 * Business logic:
 * 1. Use regex to match function signatures (supports TS, JS, Rust, Vue <script> blocks)
 * 2. For each match, track brace depth to find the function body end
 * 3. Handle string literals and comments to avoid false brace counts
 * 4. Return array of { name, body, startLine, endLine }
 */
function extractFunctions(source, isRust) {
  const functions = [];

  let patterns;
  if (isRust) {
    patterns = [
      /(?:pub\s+)?(?:async\s+)?fn\s+(\w+)\s*(?:<[^>]*>)?\s*\([^)]*\)/g
    ];
  } else {
    patterns = [
      /(?:export\s+)?(?:async\s+)?function\s+(\w+)\s*(?:<[^>]*>)?\s*\([^)]*\)/g,
      /(?:const|let)\s+(\w+)\s*=\s*(?:async\s+)?\([^)]*\)\s*(?::\s*[^=]*?)?\s*=>/g,
      /(?:const|let)\s+(\w+)\s*=\s*(?:async\s+)?function/g,
      /(?:async\s+)?(\w+)\s*\([^)]*\)\s*(?::\s*[^{]*?)?\s*\{/g,
    ];
  }

  for (const pattern of patterns) {
    let match;
    while ((match = pattern.exec(source)) !== null) {
      const name = match[1];
      const matchEnd = match.index + match[0].length;

      const braceStart = source.indexOf('{', matchEnd - 1);
      if (braceStart === -1) continue;

      const body = extractBody(source, braceStart);
      if (!body) continue;

      const startLine = source.slice(0, match.index).split('\n').length;
      const endLine = source.slice(0, braceStart + body.length + 1).split('\n').length;

      const existing = functions.find(f => f.name === name && f.startLine === startLine);
      if (!existing) {
        functions.push({ name, body, startLine, endLine });
      }
    }
  }

  functions.sort((a, b) => a.startLine - b.startLine);

  const deduped = [];
  for (const fn of functions) {
    const overlap = deduped.find(
      d => fn.startLine >= d.startLine && fn.endLine <= d.endLine && fn.name !== d.name
    );
    if (!overlap) {
      deduped.push(fn);
    }
  }

  return deduped;
}

function extractBody(source, braceStart) {
  let depth = 0;
  let inString = false;
  let stringChar = '';
  let inLineComment = false;
  let inBlockComment = false;

  for (let i = braceStart; i < source.length; i++) {
    const ch = source[i];
    const next = source[i + 1];

    if (inLineComment) {
      if (ch === '\n') inLineComment = false;
      continue;
    }

    if (inBlockComment) {
      if (ch === '*' && next === '/') {
        inBlockComment = false;
        i++;
      }
      continue;
    }

    if (inString) {
      if (ch === '\\') { i++; continue; }
      if (ch === stringChar) inString = false;
      continue;
    }

    if (ch === '/' && next === '/') { inLineComment = true; i++; continue; }
    if (ch === '/' && next === '*') { inBlockComment = true; i++; continue; }

    if (ch === '"' || ch === '\'' || ch === '`') {
      inString = true;
      stringChar = ch;
      continue;
    }

    if (ch === '{') depth++;
    if (ch === '}') {
      depth--;
      if (depth === 0) {
        return source.slice(braceStart + 1, i);
      }
    }
  }

  return null;
}

/**
 * Calculate McCabe cyclomatic complexity for a function body
 *
 * McCabe formula: M = decision_points + 1
 * Each decision point (branch/loop) adds 1 to the base complexity of 1.
 */
function calcComplexity(body, isRust) {
  const stripped = body
    .replace(/\/\/.*$/gm, '')
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .replace(/"(?:[^"\\]|\\.)*"/g, '""')
    .replace(/'(?:[^'\\]|\\.)*'/g, "''")
    .replace(/`(?:[^`\\]|\\.)*`/g, '``');

  let complexity = 1;

  if (isRust) {
    const rustPatterns = [
      /\bif\b/g,
      /\belse\s+if\b/g,
      /\bwhile\b/g,
      /\bfor\b/g,
      /\bloop\b/g,
      /=>/g,
      /&&/g,
      /\|\|/g,
    ];

    for (const pat of rustPatterns) {
      const matches = stripped.match(pat);
      if (matches) complexity += matches.length;
    }

    const elseIfCount = (stripped.match(/\belse\s+if\b/g) || []).length;
    complexity -= elseIfCount;
  } else {
    const tsPatterns = [
      /\bif\s*\(/g,
      /\belse\s+if\s*\(/g,
      /\bfor\s*\(/g,
      /\bwhile\s*\(/g,
      /\bdo\s*\{/g,
      /\bcase\s+/g,
      /\bcatch\s*\(/g,
      /\?(?!=)/g,
      /&&/g,
      /\|\|/g,
    ];

    for (const pat of tsPatterns) {
      const matches = stripped.match(pat);
      if (matches) complexity += matches.length;
    }

    const elseIfCount = (stripped.match(/\belse\s+if\s*\(/g) || []).length;
    complexity -= elseIfCount;
  }

  return complexity;
}

function countLines(body) {
  return body.split('\n').filter(line => {
    const trimmed = line.trim();
    return trimmed.length > 0 && trimmed !== '}' && trimmed !== '};';
  }).length;
}

// Main
let sourceForParsing = content;
if (ext === '.vue') {
  const scriptMatch = content.match(/<script[^>]*>([\s\S]*?)<\/script>/);
  if (scriptMatch) {
    sourceForParsing = scriptMatch[1];
  }
}

const functions = extractFunctions(sourceForParsing, isRust);

if (functions.length === 0) {
  console.log('No functions found in ' + filePath);
  process.exit(0);
}

const results = functions
  .filter(fn => !targetFunction || fn.name === targetFunction)
  .map(fn => {
    const complexity = calcComplexity(fn.body, isRust);
    const lines = countLines(fn.body);
    const needsComment = lines > 20 || complexity > 5;
    return { ...fn, complexity, lines, needsComment };
  });

if (results.length === 0 && targetFunction) {
  console.log(`Function "${targetFunction}" not found in ${filePath}`);
  process.exit(1);
}

console.log(`File: ${filePath}`);
console.log('');
console.log('Function'.padEnd(35) + 'Lines'.padEnd(8) + 'Complexity'.padEnd(13) + 'Comment Required');
console.log('─'.repeat(75));

for (const r of results) {
  const flag = r.needsComment ? '⚠ YES' : '  no';
  const name = r.name.length > 33 ? r.name.slice(0, 30) + '...' : r.name;
  console.log(
    name.padEnd(35) +
    String(r.lines).padEnd(8) +
    String(r.complexity).padEnd(13) +
    flag
  );
}

console.log('');
const needComment = results.filter(r => r.needsComment);
if (needComment.length > 0) {
  console.log(`⚠ ${needComment.length} function(s) require documentation comments.`);
} else {
  console.log('✓ All functions are within thresholds.');
}
