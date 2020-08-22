import {Construct, Stack, StackProps} from '@aws-cdk/core';
import {DatabaseStack} from './database-stack';
import {Parameters} from './parameters';
import {QueueStack} from './queue-stack';

type Props = Parameters & StackProps;

export class ScalableEmail extends Stack {
  constructor(scope: Construct, id: string, props: Props) {
    super(scope, id, props);
    // Get the stage from passed props
    const {stage} = props;
    // Set up stack level tags
    this.tags.setTag('Stage', stage);
    this.tags.setTag('Application', 'ScalableEmail');
    // Set up Dynamo
    new DatabaseStack(this, 'Database', {stage});
    // Set up SQS
    new QueueStack(this, 'Queue', {stage});
  }
}
