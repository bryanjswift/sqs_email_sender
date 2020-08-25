import {expect, haveResource, stringLike} from '@aws-cdk/assert';
import {App, Stack} from '@aws-cdk/core';
import {Role} from '@aws-cdk/aws-iam';
import {Function as LambdaFn} from '@aws-cdk/aws-lambda';
import {Queue as SQSQueue} from '@aws-cdk/aws-sqs';
import test from 'tape';
import {SqsHandler} from '../lib/sqs-handler-construct';

test('QueueStack', (t) => {
  const app = new App();
  const stack = new Stack(app, 'MyTestStack');
  const queue = new SQSQueue(stack, 'MyTestQueue');
  // WHEN
  const construct = new SqsHandler(stack, 'SqsHandlerConstruct', {
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
    2,
    'Should have list of dependencies containing each set of dependencies in the construct'
  );
  // It has a LambdaFn -> Role dependency
  const roleDep = construct.node.dependencies.find(
    (node) => node.source instanceof LambdaFn && node.target instanceof Role
  );
  t.ok(roleDep, 'There is a dependency from LambdaFn to Role');
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
  // Role
  try {
    // The `@aws-cdk/assert` library is written to be used with Jest. It throws
    // if the expectation fails.
    expect(stack).to(
      haveResource('AWS::IAM::Role', {
        RoleName: 'email_handler_role_test',
        ManagedPolicyArns: [
          'arn:aws:iam::aws:policy/service-role/AWSLambdaDynamoDBExecutionRole',
          'arn:aws:iam::aws:policy/service-role/AWSLambdaSQSQueueExecutionRole',
          'arn:aws:iam::aws:policy/AWSXrayWriteOnlyAccess',
        ],
      })
    );
  } catch (e) {
    t.fail('Role with policies exists', e instanceof Error ? e : undefined);
  }
  // It has an output for handler arn
  t.ok(construct.handlerArn, 'There is an output for handler ARN');
  t.is(
    construct.handlerArn.node.id,
    'HandlerArn',
    'At least according to its name'
  );
  t.end();
});
