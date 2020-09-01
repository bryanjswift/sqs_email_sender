import {expect, haveResource, stringLike} from '@aws-cdk/assert';
import {App, Stack} from '@aws-cdk/core';
import {AnyPrincipal, Role} from '@aws-cdk/aws-iam';
import {Queue as SQSQueue} from '@aws-cdk/aws-sqs';
import test from 'tape';
import {SqsHandler} from '../lib/sqs-handler-construct';

test('SqsHandlerConstruct', (t) => {
  const app = new App();
  const stack = new Stack(app, 'MyTestStack');
  const queue = new SQSQueue(stack, 'MyTestQueue');
  // WHEN
  const construct = new SqsHandler(stack, 'SqsHandlerConstruct', {
    handlerRole: new Role(stack, 'MyTestRole', {assumedBy: new AnyPrincipal()}),
    queue,
    stage: 'test',
  });
  // THEN
  t.is(
    construct.node.id,
    'SqsHandlerConstruct',
    'Stack name should match name from initialization'
  );
  t.is(
    construct.node.dependencies.length,
    1,
    'Should have list of dependencies containing each set of dependencies in the construct'
  );
  // It has a Queue child
  try {
    // The `@aws-cdk/assert` library is written to be used with Jest. It throws
    // if the expectation fails.
    expect(stack).to(
      haveResource('AWS::Lambda::Function', {
        FunctionName: 'email_handler_test',
        Runtime: 'provided',
      })
    );
  } catch (e) {
    t.fail(
      'Lambda function exists in children',
      e instanceof Error ? e : undefined
    );
  }
  // It has an event source
  try {
    // The `@aws-cdk/assert` library is written to be used with Jest. It throws
    // if the expectation fails.
    expect(stack).to(
      haveResource('AWS::Lambda::EventSourceMapping', {
        EventSourceArn: {
          'Fn::GetAtt': [stringLike('MyTestQueue*'), 'Arn'],
        },
      })
    );
  } catch (e) {
    t.fail(
      'EventSourceMapping function exists in children',
      e instanceof Error ? e : undefined
    );
  }
  // It has an output for handler arn
  t.ok(construct.fn, 'There is an output for handler ARN');
  t.is(construct.fn.node.id, 'SqsHandler', 'At least according to its name');
  t.end();
});
