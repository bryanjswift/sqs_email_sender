import {App} from '@aws-cdk/core';
import {ScalableEmail} from '../lib/scalable-email-stack';

interface Tag {
  Key: string;
  Value: string;
}

test('EmaillSenderStack', () => {
  const app = new App();
  // WHEN
  const stack = new ScalableEmail(app, 'MyTestStack', {stage: 'test'});
  // THEN
  expect(stack.stackName).toEqual('MyTestStack');
  expect(stack.tags.hasTags()).toBe(true);
  const tags: Tag[] = stack.tags.renderTags();
  expect(tags.find((tag) => tag.Key === 'Application')).toEqual({
    Key: 'Application',
    Value: 'ScalableEmail',
  });
  expect(stack.dependencies).toEqual([]);
});
