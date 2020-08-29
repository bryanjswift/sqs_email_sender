import {Construct, RemovalPolicy} from '@aws-cdk/core';
import {Role} from '@aws-cdk/aws-iam';
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
  handlerRole: Role;
  queue: IQueue;
}

export class SqsHandler extends Construct {
  readonly fn: LambdaFn;

  constructor(scope: Construct, id: string, props: Props) {
    super(scope, id);
    // Get the "Stage" being deployed
    const {handlerRole, queue, stage} = props;
    //
    const assetPath = resolve(
      join(__dirname, '..', '..', '..', 'email_lambda.zip')
    );
    if (!existsSync(assetPath)) {
      throw new Error(`${assetPath} must exist for lambda creation.`);
    }
    //
    const fn = new LambdaFn(this, 'SqsHandler', {
      code: Code.fromAsset(assetPath),
      currentVersionOptions: {
        removalPolicy: RemovalPolicy.RETAIN,
      },
      description: 'Process messages from SQS to send emails.',
      environment: {
        DYNAMO_TABLE: `email_db_${stage}`,
        QUEUE_URL: queue.queueUrl,
      },
      role: handlerRole,
      functionName: `email_handler_${stage}`,
      handler: 'doesnt.matter',
      runtime: Runtime.PROVIDED,
      tracing: Tracing.ACTIVE,
    });
    fn.currentVersion.addAlias(`live_${stage}`);
    fn.currentVersion.addEventSource(new SqsEventSource(queue));
    this.fn = fn;
  }
}
