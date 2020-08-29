import {Aws, CfnOutput, Construct, Stack, StackProps} from '@aws-cdk/core';
import {AccountPrincipal, Role, ServicePrincipal} from '@aws-cdk/aws-iam';
import {DatabaseStack} from './database-stack';
import {Parameters} from './parameters';
import {QueueStack} from './queue-stack';
import {SqsHandler} from './sqs-handler-construct';

type Props = Omit<Parameters, 'stackId'> & StackProps;

export class ScalableEmail extends Stack {
  readonly handlerArn: CfnOutput;
  readonly handlerVersion: CfnOutput;
  readonly queueUrl: CfnOutput;
  readonly tableName: CfnOutput;

  constructor(scope: Construct, id: string, props: Props) {
    super(scope, id, props);
    // Get the stage from passed props
    const {stage} = props;
    // Set up stack level tags
    this.tags.setTag('Stage', stage);
    this.tags.setTag('Application', 'ScalableEmail');
    // Set up Dynamo
    const databaseConstruct = new DatabaseStack(this, 'Database', {
      stage,
    });
    // Set up SQS
    const queueConstruct = new QueueStack(this, 'Queue', {stage});
    // Create AWS Role to use with function
    const handlerRole = new Role(this, 'SqsHandlerRole', {
      assumedBy: new ServicePrincipal('lambda.amazonaws.com'),
      description:
        'Allows Lambda functions to call AWS services in order to process SQS messages as emails.',
      roleName: `${id}HandlerRole`,
    });
    databaseConstruct.grantDecrypt(handlerRole);
    queueConstruct.grantDecrypt(handlerRole);
    // Set up lambda
    const handlerConstruct = new SqsHandler(this, 'Handler', {
      handlerRole,
      queue: queueConstruct.queue,
      stage,
    });
    // Dependencies
    handlerConstruct.node.addDependency(databaseConstruct);
    handlerConstruct.node.addDependency(queueConstruct);
    handlerConstruct.node.addDependency(handlerRole);
    // Add policy to allow key to be used for encryption by account
    // TODO: This should be removed, it is primarily for testing
    const accountPrincipal = new AccountPrincipal(Aws.ACCOUNT_ID);
    databaseConstruct.grantEncrypt(accountPrincipal);
    queueConstruct.grantEncrypt(accountPrincipal);
    // Assign Outputs
    this.handlerArn = new CfnOutput(this, 'HandlerArn', {
      value: handlerConstruct.fn.functionArn,
      description: 'ARN of the Handler function',
    });
    this.handlerVersion = new CfnOutput(this, 'HandlerVersion', {
      value: handlerConstruct.fn.currentVersion.version,
      description: 'Most recently deployed version of the Handler function',
    });
    this.queueUrl = new CfnOutput(this, 'QueueUrl', {
      value: queueConstruct.queue.queueUrl,
      description: 'URL of the Email Messages SQS Queue.',
    });
    this.tableName = new CfnOutput(this, 'DatabaseTable', {
      value: databaseConstruct.table.tableName,
      description: 'Name of the Dynamo Table storing message data.',
    });
  }
}
