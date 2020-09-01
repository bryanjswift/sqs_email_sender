import {CfnOutput, Construct, Stack, StackProps} from '@aws-cdk/core';
import {
  ArnPrincipal,
  ManagedPolicy,
  Role,
  ServicePrincipal,
} from '@aws-cdk/aws-iam';
import {DatabaseConstruct} from './database-construct';
import {Parameters} from './parameters';
import {QueueConstruct} from './queue-construct';
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
    const databaseConstruct = new DatabaseConstruct(this, 'Database', {
      stage,
    });
    // Set up SQS
    const queueConstruct = new QueueConstruct(this, 'Queue', {stage});
    // Create AWS Role to use with function
    const handlerRole = new Role(this, 'SqsHandlerRole', {
      assumedBy: new ServicePrincipal('lambda.amazonaws.com'),
      description:
        'Allows Lambda functions to call AWS services in order to process SQS messages as emails.',
      managedPolicies: [
        ManagedPolicy.fromManagedPolicyArn(
          this,
          'LambdaExecPolicy',
          'arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole'
        ),
      ],
      roleName: `${id}HandlerRole`,
    });
    databaseConstruct.grantReadWriteData(handlerRole);
    queueConstruct.grantConsumeMessages(handlerRole);
    // Set up lambda
    const handlerConstruct = new SqsHandler(this, 'Handler', {
      handlerRole,
      queue: queueConstruct.queue,
      stage,
    });
    // Dependencies
    handlerConstruct.addDependency(
      databaseConstruct,
      queueConstruct,
      handlerRole
    );
    // Add policy to allow key to be used for encryption by account
    // TODO: This should be removed, it is primarily for testing
    const userPrincipal = new ArnPrincipal(
      'arn:aws:iam::161895662097:user/bryanjswift'
    );
    databaseConstruct.grantWriteData(userPrincipal);
    queueConstruct.grantSendMessages(userPrincipal);
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
