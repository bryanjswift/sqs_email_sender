import {expect, haveResource} from '@aws-cdk/assert';
import {App, Stack} from '@aws-cdk/core';
import {Alias, Key} from '@aws-cdk/aws-kms';
import {Queue as SQSQueue} from '@aws-cdk/aws-sqs';
import test from 'tape';
import {QueueStack} from '../lib/queue-stack';

test('QueueStack', (t) => {
  const app = new App();
  const stack = new Stack(app, 'MyTestStack');
  // WHEN
  const construct = new QueueStack(stack, 'QueueConstruct', {stage: 'test'});
  // THEN
  t.is(
    construct.node.id,
    'QueueConstruct',
    'Stack name should match name from initialization'
  );
  t.is(
    construct.node.dependencies.length,
    2,
    'Should have list of dependencies containing each set of dependencies in the construct'
  );
  // It has an Alias -> Key dependency
  const aliasDep = construct.node.dependencies.find(
    (node) => node.source instanceof Alias
  );
  t.ok(aliasDep, 'There is a dependency starting with Alias');
  t.true(
    aliasDep?.target instanceof Key,
    'There is a dependency from Alias to Key'
  );
  // It has a SQS -> Alias dependency
  const queueDep = construct.node.dependencies.find(
    (node) => node.source instanceof SQSQueue
  );
  t.ok(queueDep, 'There is a dependency starting with the SQS Queue');
  t.true(
    queueDep?.target instanceof Alias,
    'There is a dependency from SQS to Alias'
  );
  // It has a Queue child
  try {
    // The `@aws-cdk/assert` library is written to be used with Jest. It throws
    // if the expectation fails.
    expect(stack).to(
      haveResource('AWS::SQS::Queue', {
        QueueName: 'email_queue_test',
      })
    );
  } catch (e) {
    t.fail('SQS Queue exists in children', e instanceof Error ? e : undefined);
  }
  // It has a queue url output
  t.ok(construct.queue, 'There is a property for queue created queue');
  t.is(construct.queue.node.id, 'Messages', 'At least according to its name');
  t.end();
});
