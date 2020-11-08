import test from 'tape';
import {format} from 'util';

let verbose = process.env.TAPE_VERBOSE === 'true';

setupReporter();

/**
 * Creates a readable stream from `tape` and manages writing results to stdout.
 */
function setupReporter() {
  let context: string[] = [];
  let failures: FailureWithContext[] = [];
  let results: ResultSummary = {
    failCount: 0,
    okCount: 0,
  };

  const stream = test.createStream({objectMode: true});
  stream.on('data', (chunk: unknown) => {
    if (verbose && isCommentLine(chunk)) {
      process.stdout.write(commentLineReport(context, chunk));
      return;
    } else if (
      !(isAssertLine(chunk) || isEndLine(chunk) || isTestLine(chunk))
    ) {
      // skip any chunk that is a comment or isn't a known type
      return;
    }

    switch (chunk.type) {
      case 'test':
        context.push(chunk.name);
        break;
      case 'assert':
        process.stdout.write(assertionLineReport(context, chunk));
        failures = chunk.ok
          ? failures
          : [...failures, {...chunk, context: context.concat([])}];
        results = {
          failCount: chunk.ok ? results.failCount : results.failCount + 1,
          okCount: chunk.ok ? results.okCount + 1 : results.okCount,
        };
        break;
      case 'end':
        context.pop();
        break;
    }
  });
  stream.on('end', () => {
    process.stdout.write(resultsSummaryReport(results));
    process.stdout.write(failureSummaryReport(failures));
  });
}

/**
 * Create a formatted string for reporting the results of an assertion/test from tape.
 * @param context at the time the `chunk` was received
 * @param chunk with data about the assertion/test
 * @returns the formatted assertion string
 */
function assertionLineReport(context: string[], chunk: Assert): string {
  if (verbose) {
    const result = chunk.ok ? 'ok' : 'not ok';
    const suffix = chunk.name;
    return format('%s: %s - %s\n', result, makeContext(context), suffix);
  } else {
    // Append a dot?
    return '.';
  }
}

/**
 * Create a string for reporting a `comment` with the associated `context`.
 * @param context at the time `comment` was received
 * @param comment to be reported
 * @returns the formatted comment string
 */
function commentLineReport(context: string[], comment: Comment): string {
  if (verbose) {
    return format('# %s %s\n', makeContext(context, ' -'), comment);
  } else {
    return '';
  }
}

/**
 * Create a string with newlines that is a "failure report" based on the given
 * `failures`.
 * @param failures containing the test failures to report upon
 * @returns the formatted string
 */
function failureSummaryReport(failures: FailureWithContext[]): string {
  let out = failures.length === 0 ? '' : '\n';
  if (verbose && failures.length > 0) {
    // Print an empty line
    out += '** Test Failure Summary **\n';
    out += '\n';
  }

  const pwd = new RegExp(`^${process.cwd()}`);
  for (const failure of failures) {
    const {context, id, file, name} = failure;
    out += format('%s\n', file.replace(pwd, '.'));
    out += format('↳ %s\n', makeContext(context));
    out += format('↳ Assertion #%d - %s\n', id, name);
  }
  return out;
}

/**
 * Create a formatted summary report with newlines.
 * @param results containing the data to summarize
 * @returns the formatted string
 */
function resultsSummaryReport(results: ResultSummary): string {
  let out = verbose ? '\n' : '\n\n';
  out += `not ok: ${results.failCount}\n`;
  out += `ok: ${results.okCount}\n`;
  return out;
}

function isAssumedType<T extends Record<string, unknown>>(x: unknown): x is T {
  return typeof x === 'object' && x !== null;
}

function isAssertLine(line: unknown): line is Assert {
  return isAssumedType(line) && 'type' in line && line.type === 'assert';
}

function isCommentLine(line: unknown): line is Comment {
  return typeof line === 'string';
}

function isEndLine(line: unknown): line is End {
  return isAssumedType(line) && 'type' in line && line.type === 'end';
}

function isTestLine(line: unknown): line is Test {
  return isAssumedType(line) && 'type' in line && line.type === 'test';
}

function makeContext(context: string[], suffix = ''): string {
  return `${context.join(' - ')}${suffix}`;
}

type Assert = AssertSuccess | AssertFailure;

type FailureWithContext = AssertFailure & WithContext;

interface BaseAssert {
  type: 'assert';
  id: number;
  skip?: boolean;
  todo: boolean;
  name: string;
  operator: string;
  test: number;
}

interface AssertWithValue extends BaseAssert {
  actual: unknown;
  expected: unknown;
}

interface AssertSuccess extends AssertWithValue {
  ok: true;
}

interface AssertFailure extends AssertWithValue {
  ok: false;
  error: Error;
  functionName: string;
  file: string;
  line: number;
  column: number;
  at: string;
}

interface End {
  type: 'end';
  test: number;
}

interface ResultSummary {
  failCount: number;
  okCount: number;
}

interface Test {
  type: 'test';
  name: string;
  id: number;
  skip: boolean;
  todo: boolean;
}

interface WithContext {
  context: string[];
}

type Comment = string;
