import {Construct, Stack, StackProps} from '@aws-cdk/core';
import {DatabaseStack} from './database-stack';
import {Parameters} from './parameters';
import {QueueStack} from './queue-stack';
import {SqsHandler} from './sqs-handler-construct';

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
    const databaseConstruct = new DatabaseStack(this, 'Database', {stage});
    // Set up SQS
    const queueConstruct = new QueueStack(this, 'Queue', {stage});
    // Set up lambda
    const handlerConstruct = new SqsHandler(this, 'Handler', {
      queue: queueConstruct.queue,
      stage,
    });
    // Dependencies
    handlerConstruct.node.addDependency(databaseConstruct);
  }
}
