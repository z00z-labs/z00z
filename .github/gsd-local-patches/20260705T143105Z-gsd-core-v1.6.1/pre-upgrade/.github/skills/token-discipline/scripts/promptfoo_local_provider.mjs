function extractTask(prompt) {
  const match = prompt.match(/Task:\s*([\s\S]*)$/i);
  return (match ? match[1] : prompt).trim();
}

function compactLine(text) {
  return text.replace(/\s+/g, ' ').trim();
}

function buildDocsAnswer() {
  return compactLine('Use AGENTS.md at repo root for always-loaded rules. Put reusable workflows in .github/skills/<name>/SKILL.md and validate the skill after edits.');
}

function buildTestAnswer() {
  return compactLine('Re-run the exact failing test first, inspect the first mismatch, and only widen scope if that local check does not explain the failure.');
}

function buildArchitectureAnswer() {
  return [
    'recommendation: "Use AGENTS.md for baseline rules and skills for task-specific workflows."',
    'why:',
    '  - "AGENTS.md stays always loaded."',
    'tradeoffs:',
    '  - "Weak skill descriptions hurt discovery."',
    'next_step: "Add one validated skill with a local eval path."',
  ].join('\n');
}

function buildReviewAnswer() {
  return [
    'findings:',
    '  - severity: high',
    '    file: src/example.rs',
    '    issue: review output drift hides the first actionable bug',
    '    fix: put the concrete fix beside the finding and drop filler prose',
  ].join('\n');
}

function buildCompressionAnswer() {
  return compactLine('Use AGENTS.md at repo root, validate the skill after edits, prefer a local offline eval path, and debug from the first failing local check.');
}

function buildDefaultAnswer(task) {
  const normalized = compactLine(task);
  return `answer: "${normalized.slice(0, 140)}"`;
}

export default class TokenDisciplinePromptfooProvider {
  constructor(options = {}) {
    this.options = options;
  }

  id() {
    return this.options.id || 'token-discipline-local';
  }

  async callApi(prompt) {
    const task = extractTask(prompt);
    const lower = task.toLowerCase();
    let output;

    if (lower.includes('compress a long repository setup prompt') || lower.includes('offline evals')) {
      output = buildCompressionAnswer();
    } else if (lower.includes('agents.md') || lower.includes('skill.md')) {
      output = buildDocsAnswer();
    } else if (lower.includes('failing test') || (lower.includes('test') && lower.includes('debug'))) {
      output = buildTestAnswer();
    } else if (lower.includes('architecture') || lower.includes('tradeoffs') || lower.includes('recommend one')) {
      output = buildArchitectureAnswer();
    } else if (lower.includes('review') && lower.includes('fix')) {
      output = buildReviewAnswer();
    } else {
      output = buildDefaultAnswer(task);
    }

    return { output };
  }
}