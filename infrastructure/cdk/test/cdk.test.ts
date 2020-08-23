import {App} from '@aws-cdk/core';
import test from 'tape';
import {ScalableEmail} from '../lib/scalable-email-stack';

interface Tag {
  Key: string;
  Value: string;
}

test('EmaillSenderStack', (t) => {
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
  t.deepEqual(
    stack.dependencies,
    [],
    'Should have an empty list of dependencies'
  );
  t.is(
    stack.node.children.length,
    2,
    'Should have child constructs for components'
  );
  t.end();
});
