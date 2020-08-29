import {expect, haveResource} from '@aws-cdk/assert';
import {App} from '@aws-cdk/core';
import test from 'tape';
import {ScalableEmail} from '../lib/scalable-email-stack';

interface Tag {
  Key: string;
  Value: string;
}

test('ScalableEmail', (t) => {
  const app = new App();
  // WHEN
  const stack = new ScalableEmail(app, 'MyTestStack', {stage: 'test'});
  // THEN
  t.is(
    stack.stackName,
    'MyTestStack',
    'Stack name should match name from initialization'
  );
  t.true(stack.tags.hasTags(), 'Stack should be assigned tags');
  const tags: Tag[] = stack.tags.renderTags();
  t.is(
    tags.find((tag) => tag.Key === 'Application')?.Value,
    'ScalableEmail',
    'Should have an Application tag identifying itself.'
  );
  t.is(
    tags.find((tag) => tag.Key === 'Stage')?.Value,
    'test',
    'Should have a Stage tag with the passed `stage`'
  );
  t.isEquivalent(
    stack.dependencies,
    [],
    'Should have an empty list of dependencies'
  );
  t.is(
    stack.node.children.length,
    9, // 4 for constructs, 1 to track dependencies, 4 for outputs
    'Should have child constructs for components'
  );
  t.isEquivalent(
    stack.node.children.map((child) => child.node.id),
    [
      'Database',
      'Queue',
      'SqsHandlerRole',
      'Handler',
      'AssetParameters',
      'HandlerArn',
      'HandlerVersion',
      'QueueUrl',
      'DatabaseTable',
    ]
  );
  // Role
  try {
    // The `@aws-cdk/assert` library is written to be used with Jest. It throws
    // if the expectation fails.
    expect(stack).to(
      haveResource('AWS::IAM::Role', {
        RoleName: 'MyTestStackHandlerRole',
      })
    );
  } catch (e) {
    t.fail('Role with policies exists', e instanceof Error ? e : undefined);
  }
  t.end();
});
