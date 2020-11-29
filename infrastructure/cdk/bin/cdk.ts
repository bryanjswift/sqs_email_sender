#!/usr/bin/env node
import 'source-map-support/register';
import {App} from '@aws-cdk/core';
import {ScalableEmail} from '../lib/scalable-email-stack';

export type StackMap = Record<string, ScalableEmail | undefined>;

export const STAGES = ['Local', 'Test', 'Staging', 'Production'];

const app = new App();
export const stacks = STAGES.reduce<StackMap>((map, stage) => {
  const stack = new ScalableEmail(app, `ScalableEmail${stage}`, {
    description:
      'Stack to enable ansynchronous processing of a large volume of email. Email information is stored in DynamoDB and the send trigger is pushed into SQS.',
    stage: stage.toLowerCase(),
  });
  return {
    ...map,
    [stage]: stack,
  };
}, {});
