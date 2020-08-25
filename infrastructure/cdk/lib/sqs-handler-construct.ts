import {Construct, CfnOutput, RemovalPolicy} from '@aws-cdk/core';
import {ManagedPolicy, Role, ServicePrincipal} from '@aws-cdk/aws-iam';
import {
  Code,
  Function as LambdaFn,
  Runtime,
  Tracing,
} from '@aws-cdk/aws-lambda';
import {SqsEventSource} from '@aws-cdk/aws-lambda-event-sources';
import {IQueue} from '@aws-cdk/aws-sqs';
import {existsSync} from 'fs';
import {join, resolve} from 'path';
import {Parameters} from './parameters';

interface Props extends Parameters {
  queue: IQueue;
}

export class SqsHandler extends Construct {
  readonly handlerArn: CfnOutput;
  readonly handlerVersion: CfnOutput;
  readonly role: Role;

  constructor(scope: Construct, id: string, props: Props) {
    super(scope, id);
    // Get the "Stage" being deployed
    const {queue, stage} = props;
    //
    const assetPath = resolve(
      join(__dirname, '..', '..', '..', 'email_lambda.zip')
    );
    if (!existsSync(assetPath)) {
      throw new Error(`${assetPath} must exist for lambda creation.`);
    }
    // Create AWS Role to use with function
    const role = new Role(this, 'SqsHandlerRole', {
      assumedBy: new ServicePrincipal('lambda.amazonaws.com'),
      description:
        'Allows Lambda functions to call AWS services in order to process SQS messages as emails.',
      managedPolicies: [
        ManagedPolicy.fromManagedPolicyArn(
          this,
          'DynamoDB',
          'arn:aws:iam::aws:policy/service-role/AWSLambdaDynamoDBExecutionRole'
        ),
        ManagedPolicy.fromManagedPolicyArn(
          this,
          'SQS',
          'arn:aws:iam::aws:policy/service-role/AWSLambdaSQSQueueExecutionRole'
        ),
        ManagedPolicy.fromManagedPolicyArn(
          this,
          'XRay',
          'arn:aws:iam::aws:policy/AWSXrayWriteOnlyAccess'
        ),
      ],
      roleName: `email_handler_role_${stage}`,
    });
    //
    const fn = new LambdaFn(this, 'SqsHandler', {
      code: Code.fromAsset(assetPath),
      currentVersionOptions: {
        removalPolicy: RemovalPolicy.RETAIN,
      },
      description: 'Process messages from SQS to send emails.',
      role,
      functionName: `email_handler_${stage}`,
      handler: 'doesnt.matter',
      runtime: Runtime.PROVIDED,
      tracing: Tracing.ACTIVE,
    });
    fn.currentVersion.addAlias(`live_${stage}`);
    fn.addEventSource(new SqsEventSource(queue));
    fn.node.addDependency(role);
    this.handlerArn = new CfnOutput(this, 'HandlerArn', {
      value: fn.functionArn,
      description: 'ARN of the Handler function',
    });
    this.handlerVersion = new CfnOutput(this, 'HandlerVersion', {
      value: fn.currentVersion.version,
      description: 'Most recently deployed version of the Handler function',
    });
    this.role = role;
  }
}
