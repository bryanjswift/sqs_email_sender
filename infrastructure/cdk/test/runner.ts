import test from 'tape';
import {format} from 'util';

const verbosity: VerbosityLevel = getVerbosity(process.env.TAPE_VERBOSITY);

setupReporter();

/**
 * Creates a readable stream from `tape` and manages writing results to stdout.
 */
function setupReporter(): void {
  let context: readonly string[] = [];
  let failures: readonly FailureWithContext[] = [];
  let results: ResultSummary = {
    failCount: 0,
    okCount: 0,
  };

  const stream = test.createStream({objectMode: true});
  stream.on('data', (chunk: unknown) => {
    if (verbosity === 'full' && isCommentLine(chunk)) {
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
        context = context.concat(chunk.name);
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
        context = context.slice(0, -1);
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
function assertionLineReport(
  context: readonly string[],
  chunk: Assert
): string {
  if (verbosity === 'full') {
    const result = chunk.ok ? 'ok' : 'not ok';
    const suffix = chunk.name;
    return format('%s: %s - %s\n', result, makeContext(context), suffix);
  } else if (verbosity === 'lite') {
    // Append a dot?
    return '.';
  } else {
    return '';
  }
}

/**
 * Create a string for reporting a `comment` with the associated `context`.
 * @param context at the time `comment` was received
 * @param comment to be reported
 * @returns the formatted comment string
 */
function commentLineReport(
  context: readonly string[],
  comment: Comment
): string {
  if (verbosity === 'full') {
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
function failureSummaryReport(failures: readonly FailureWithContext[]): string {
  let out = failures.length === 0 ? '' : '\n';
  if ((verbosity === 'full' || verbosity === 'lite') && failures.length > 0) {
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
  let out = verbosity === 'summary' ? '' : verbosity === 'full' ? '\n' : '\n\n';
  out += `not ok: ${results.failCount}\n`;
  out += `ok: ${results.okCount}\n`;
  return out;
}

/**
 * Assume the given value is a given "object" type with string keys if it is an
 * object.
 * @param x to be assumed
 * @returns `true` if `x` exists and is not `null`
 */
function isAssumedType<T extends Record<string, unknown>>(x: unknown): x is T {
  return typeof x === 'object' && x !== null;
}

/**
 * Check if the given `line` is of the `Assert` type.
 * @param line to be checked
 * @returns `true` if `line` has the `'assert'` type value
 */
function isAssertLine(line: unknown): line is Assert {
  return isAssumedType(line) && 'type' in line && line.type === 'assert';
}

/**
 * Check if the given `line` is of the `Comment` type
 * @param line to be checked
 * @returns `true` if `line` is a comment.
 */
function isCommentLine(line: unknown): line is Comment {
  return typeof line === 'string';
}

/**
 * Check if the given `line` is of the `End` type
 * @param line to be checked
 * @returns `true` if `line` has the `'end'` type value.
 */
function isEndLine(line: unknown): line is End {
  return isAssumedType(line) && 'type' in line && line.type === 'end';
}

/**
 * Check if the given `line` is of the `Test` type.
 * @param line to be checked
 * @returns `true` if `line` has the `'test'` type value
 */
function isTestLine(line: unknown): line is Test {
  return isAssumedType(line) && 'type' in line && line.type === 'test';
}

/**
 * Create a context string from an a context "stack".
 * @param context stack to include in the string
 * @param suffix to append to the context
 * @returns a combination of context and `suffix`
 */
function makeContext(context: readonly string[], suffix = ''): string {
  return `${context.join(' - ')}${suffix}`;
}

/**
 * Get the `VerbosityLevel` based on the given `string` value.
 * @param verbosity to turn into `VerbosityLevel`
 * @returns the level
 */
function getVerbosity(verbosity?: string): VerbosityLevel {
  switch (verbosity) {
    case 'full': // Intentional fall through
    case 'lite': // Intentional fall through
    case 'summary':
      return verbosity;
    default:
      return 'lite';
  }
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

type VerbosityLevel = 'full' | 'lite' | 'summary';
